#!/usr/bin/env python3
import json
import sys
import random


def generate_mode_graph(n, mode):
    """Generate a single mode graph with n nodes."""
    nodes = []
    edges = []

    # Calculate grid dimensions (approximately square)
    cols = int(n ** 0.5)
    rows = (n + cols - 1) // cols
    spacing = 5.0

    # Generate nodes in grid with small random offset
    node_id = 0
    for row in range(rows):
        for col in range(cols):
            if node_id >= n:
                break
            x = col * spacing + random.uniform(-0.5, 0.5)
            z = row * spacing + random.uniform(-0.5, 0.5)
            nodes.append({
                "id": node_id,
                "position": [x, z],
                "node_type": "Intersection",
                "physical_attributes": [],
                "turn_restrictions": []
            })
            node_id += 1

    # Mode-specific facility types
    facility_types_by_mode = {
        "Bike": ["ProtectedLane", "BufferedLane", "SharedLane"],
        "Walk": ["Sidewalk", "SharedUsePath", "Trail"],
        "Transit": ["BusLane", "Rail", "BRT"],
        "Car": ["Highway", "Arterial", "LocalStreet"],
    }

    facility_types = facility_types_by_mode.get(mode, ["Generic"])

    # Generate edges (connect to right and down neighbors)
    edge_id = 0
    for i in range(len(nodes)):
        row = i // cols
        col = i % cols

        # Connect to right neighbor
        if col < cols - 1 and i + 1 < len(nodes):
            edges.append({
                "id": edge_id,
                "from_node": i,
                "to_node": i + 1,
                "facility_type": random.choice(facility_types),
                "physical_attributes": []
            })
            edge_id += 1

        # Connect to down neighbor
        if row < rows - 1 and i + cols < len(nodes):
            edges.append({
                "id": edge_id,
                "from_node": i,
                "to_node": i + cols,
                "facility_type": random.choice(facility_types),
                "physical_attributes": []
            })
            edge_id += 1

    return {
        "mode": mode,
        "nodes": nodes,
        "edges": edges
    }


def generate_network(n, modes=None):
    """Generate a network with multiple mode graphs."""
    if modes is None:
        modes = ["Bike", "Walk"]  # Default: bike and walk

    graphs = {}
    for mode in modes:
        graphs[mode] = generate_mode_graph(n, mode)

    return {"graphs": graphs}


if __name__ == "__main__":
    n = int(sys.argv[1]) if len(sys.argv) > 1 else 25

    # Parse modes from command line (comma-separated)
    if len(sys.argv) > 2:
        modes = sys.argv[2].split(',')
    else:
        modes = ["Bike", "Walk", "Transit"]  # Default modes

    network = generate_network(n, modes)
    print(json.dumps(network, indent=2))
