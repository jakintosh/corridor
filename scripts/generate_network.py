#!/usr/bin/env python3
import json
import random
import sys
from typing import Dict, List, Sequence, Tuple

import numpy as np
from scipy.spatial import Voronoi


# Tweakable percentages for pruning steps
NODE_REMOVAL_FRACTION = 0.10
EDGE_REMOVAL_FRACTION = 0.20


def generate_city_points(num_points: int, width: float, height: float, iterations: int = 2) -> np.ndarray:
    """Generate seed points with a gentle center bias then smooth with Lloyd relaxation."""
    points: List[List[float]] = []
    attempts = 0
    max_attempts = num_points * 50
    while len(points) < num_points and attempts < max_attempts:
        attempts += 1
        x = np.random.uniform(0, width)
        y = np.random.uniform(0, height)
        cx, cy = width / 2, height / 2
        dist = np.sqrt((x - cx) ** 2 + (y - cy) ** 2)
        max_dist = np.sqrt(cx**2 + cy**2)
        norm_dist = (dist / max_dist) * 0.7
        prob = (1 - norm_dist) ** 3
        if np.random.rand() < prob:
            points.append([x, y])

    pts = np.array(points)
    for _ in range(iterations):
        vor = Voronoi(pts)
        relaxed_points = []
        for idx, region_index in enumerate(vor.point_region):
            region = vor.regions[region_index]
            if -1 in region or len(region) == 0:
                relaxed_points.append(pts[idx])
                continue
            vertices = np.array([vor.vertices[i] for i in region])
            centroid = np.mean(vertices, axis=0)
            if 0 <= centroid[0] <= width and 0 <= centroid[1] <= height:
                relaxed_points.append(centroid)
            else:
                relaxed_points.append(pts[idx])
        pts = np.array(relaxed_points)
    return pts


def build_base_graph(points: np.ndarray, width: float, height: float) -> Tuple[List[Dict], List[Tuple[int, int]]]:
    """Convert Voronoi ridges into nodes/edges we can reuse across modes."""
    vor = Voronoi(points)
    nodes: List[Dict] = []
    base_edges: List[Tuple[int, int]] = []
    node_mapping: Dict[Tuple[float, float], int] = {}
    edge_seen = set()

    def ensure_node(coord: Tuple[float, float]) -> int:
        if coord not in node_mapping:
            node_id = len(nodes)
            node_mapping[coord] = node_id
            nodes.append(
                {
                    "id": node_id,
                    "position": [coord[0], coord[1]],
                    "node_type": "Intersection",
                    "physical_attributes": [],
                    "turn_restrictions": [],
                }
            )
        return node_mapping[coord]

    for p1_idx, p2_idx in vor.ridge_vertices:
        if p1_idx == -1 or p2_idx == -1:
            continue
        p1 = vor.vertices[p1_idx]
        p2 = vor.vertices[p2_idx]
        if not (0 <= p1[0] <= width and 0 <= p1[1] <= height):
            continue
        if not (0 <= p2[0] <= width and 0 <= p2[1] <= height):
            continue
        n1 = (round(float(p1[0]), 2), round(float(p1[1]), 2))
        n2 = (round(float(p2[0]), 2), round(float(p2[1]), 2))
        if n1 == n2:
            continue
        edge_key = tuple(sorted((n1, n2)))
        if edge_key in edge_seen:
            continue
        edge_seen.add(edge_key)
        node_a = ensure_node(n1)
        node_b = ensure_node(n2)
        base_edges.append((node_a, node_b))

    return nodes, base_edges


def knock_out_edges(base_edges: Sequence[Tuple[int, int]], removal_fraction: float) -> List[Tuple[int, int]]:
    """Remove a fraction of edges to shape a mode-specific graph."""
    if not base_edges or removal_fraction <= 0:
        return list(base_edges)
    edge_indices = list(range(len(base_edges)))
    remove_count = min(len(base_edges), int(len(base_edges) * removal_fraction))
    to_remove = set(random.sample(edge_indices, remove_count)) if remove_count else set()
    return [edge for idx, edge in enumerate(base_edges) if idx not in to_remove]


def prune_nodes_for_mode(nodes: List[Dict], base_edges: Sequence[Tuple[int, int]], removal_fraction: float):
    """Remove a fraction of nodes for a mode, along with any incident edges."""
    if not nodes or removal_fraction <= 0:
        return [dict(node) for node in nodes], list(base_edges)

    node_ids = [node["id"] for node in nodes]
    remove_count = min(len(node_ids), int(len(node_ids) * removal_fraction))
    removed_ids = set(random.sample(node_ids, remove_count)) if remove_count else set()

    remaining_nodes = [dict(node) for node in nodes if node["id"] not in removed_ids]
    remaining_edges = [edge for edge in base_edges if edge[0] not in removed_ids and edge[1] not in removed_ids]
    return remaining_nodes, remaining_edges


def reindex_nodes_and_edges(nodes: List[Dict], edges: Sequence[Tuple[int, int]]) -> Tuple[List[Dict], List[Tuple[int, int]]]:
    """Drop isolated nodes and remap node IDs so edges stay in-bounds."""
    if not edges:
        return [], []

    connected_ids = set()
    for src, dst in edges:
        connected_ids.add(src)
        connected_ids.add(dst)

    filtered_nodes = [dict(node) for node in nodes if node["id"] in connected_ids]
    id_map = {node["id"]: idx for idx, node in enumerate(filtered_nodes)}
    for node in filtered_nodes:
        node["id"] = id_map[node["id"]]

    remapped_edges = []
    for src, dst in edges:
        if src in id_map and dst in id_map:
            remapped_edges.append((id_map[src], id_map[dst]))

    return filtered_nodes, remapped_edges


def generate_mode_graph(mode: str, nodes: List[Dict], base_edges: Sequence[Tuple[int, int]]) -> Dict:
    """Build a single mode graph from the shared base topology."""
    facility_types_by_mode = {
        "Bike": ["ProtectedLane", "BufferedLane", "SharedLane"],
        "Walk": ["Sidewalk", "SharedUsePath", "Trail"],
        "Transit": ["BusLane", "Rail", "BRT"],
        "Car": ["Highway", "Arterial", "LocalStreet"],
    }
    facility_types = facility_types_by_mode.get(mode, ["Generic"])
    mode_nodes, node_pruned_edges = prune_nodes_for_mode(nodes, base_edges, NODE_REMOVAL_FRACTION)
    mode_edges_raw = knock_out_edges(node_pruned_edges, EDGE_REMOVAL_FRACTION)
    mode_nodes, mode_edges_raw = reindex_nodes_and_edges(mode_nodes, mode_edges_raw)

    edges = []
    for edge_id, (src, dst) in enumerate(mode_edges_raw):
        edges.append(
            {
                "id": edge_id,
                "from_node": src,
                "to_node": dst,
                "facility_type": random.choice(facility_types),
                "physical_attributes": [],
            }
        )

    return {"mode": mode, "nodes": mode_nodes, "edges": edges}


def generate_network(n: int, modes=None):
    """Generate a network with multiple mode graphs from one Voronoi base."""
    if modes is None:
        modes = ["Bike", "Walk"]

    map_size = max(30.0, (n ** 0.5) * 10.0)
    points = generate_city_points(n, map_size, map_size, iterations=3)
    nodes, base_edges = build_base_graph(points, map_size, map_size)

    if nodes:
        xs = [node["position"][0] for node in nodes]
        zs = [node["position"][1] for node in nodes]
        center_x = (min(xs) + max(xs)) / 2.0
        center_z = (min(zs) + max(zs)) / 2.0
        for node in nodes:
            node["position"][0] -= center_x
            node["position"][1] -= center_z

    graphs = {}
    for mode in modes:
        graphs[mode] = generate_mode_graph(mode, nodes, base_edges)

    return {"graphs": graphs}


if __name__ == "__main__":
    n = int(sys.argv[1]) if len(sys.argv) > 1 else 25
    modes = sys.argv[2].split(",") if len(sys.argv) > 2 else ["Bike", "Walk", "Transit"]
    network = generate_network(n, modes)
    print(json.dumps(network, indent=2))
