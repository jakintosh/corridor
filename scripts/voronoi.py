import numpy as np
import matplotlib.pyplot as plt
from scipy.spatial import Voronoi
import networkx as nx
import json

def generate_city_points(num_points=1000, width=100, height=100, iterations=3):
    """
    Generates organic city block centers using Lloyd's Relaxation.
    """
    points = []
    # Safety break to prevent infinite loops if constraints are too tight
    attempts = 0
    max_attempts = num_points * 50
    
    while len(points) < num_points and attempts < max_attempts:
        attempts += 1
        x = np.random.uniform(0, width)
        y = np.random.uniform(0, height)
        
        # Calculate distance from center
        cx, cy = width / 2, height / 2
        dist = np.sqrt((x - cx)**2 + (y - cy)**2)
        max_dist = np.sqrt(cx**2 + cy**2)
        
        # FIXED: "Random Circle" Issue
        # The previous formula faded to 0 exactly at the corners, creating a circle.
        # We multiply dist by 0.7 so the fade-out happens 'outside' the map,
        # keeping the corners populated but still less dense than the center.
        norm_dist = (dist / max_dist) * 0.7
        prob = (1 - norm_dist) ** 3
        
        if np.random.rand() < prob:
            points.append([x, y])
            
    points = np.array(points)

    # Lloyd's Relaxation
    for _ in range(iterations):
        vor = Voronoi(points)
        new_points = []
        for i, region_index in enumerate(vor.point_region):
            region = vor.regions[region_index]
            if -1 in region or len(region) == 0:
                continue
            
            region_verts = [vor.vertices[i] for i in region]
            region_verts = np.array(region_verts)
            centroid = np.mean(region_verts, axis=0)
            
            # Keep within bounds
            if 0 <= centroid[0] <= width and 0 <= centroid[1] <= height:
                new_points.append(centroid)
            else:
                new_points.append(points[i])
                
        points = np.array(new_points)

    return points

def build_graph_from_voronoi(points, width, height):
    """
    Converts Voronoi ridges into a NetworkX graph, filtering out-of-bounds edges.
    """
    vor = Voronoi(points)
    G = nx.Graph()
    
    for p1_idx, p2_idx in vor.ridge_vertices:
        if p1_idx == -1 or p2_idx == -1:
            continue
            
        p1 = vor.vertices[p1_idx]
        p2 = vor.vertices[p2_idx]
        
        # FIXED: "Crazy Outlier Points" Issue
        # We strictly ignore any edge where a vertex is outside the map bounds.
        # This prevents lines shooting off to infinity.
        if not (0 <= p1[0] <= width and 0 <= p1[1] <= height):
            continue
        if not (0 <= p2[0] <= width and 0 <= p2[1] <= height):
            continue
        
        n1 = (round(p1[0], 2), round(p1[1], 2))
        n2 = (round(p2[0], 2), round(p2[1], 2))
        
        # FIXED: "Literal Oval" / Artifact Issue
        # Sometimes Voronoi vertices are extremely close or identical after rounding.
        # This creates self-loops (edges from A to A) which Matplotlib/NX renders as 
        # small circles or ovals. We must skip these.
        if n1 == n2:
            continue
            
        G.add_node(n1, pos=n1)
        G.add_node(n2, pos=n2)
        G.add_edge(n1, n2)
        
    return G

def plot_city(G, points):
    plt.figure(figsize=(12, 12), facecolor='#f0f0f0')
    ax = plt.gca()
    ax.set_facecolor('#f0f0f0')
    
    plt.scatter(points[:,0], points[:,1], s=5, c='gray', alpha=0.3, label='Block Centers')
    
    pos = nx.get_node_attributes(G, 'pos')
    
    nx.draw_networkx_edges(G, pos, 
                           edge_color='#333333', 
                           width=1.5, 
                           alpha=0.8)
    
    plt.title("Organic City Layout via Voronoi Relaxation")
    plt.axis('equal')
    plt.axis('off')
    plt.show()

def export_graph_to_json(G, filename="city_graph.json"):
    """
    Exports the graph to a JSON file with nodes and edges.
    """
    # Create a mapping from coordinate tuple to integer ID for cleaner JSON
    # e.g. (10.5, 20.1) -> 1
    node_mapping = {node: i for i, node in enumerate(G.nodes())}
    
    output_data = {
        "nodes": [],
        "edges": []
    }
    
    # Serialize Nodes
    for node_pos, node_id in node_mapping.items():
        output_data["nodes"].append({
            "id": node_id,
            "x": node_pos[0],
            "y": node_pos[1]
        })
        
    # Serialize Edges
    for u, v in G.edges():
        output_data["edges"].append({
            "source": node_mapping[u],
            "target": node_mapping[v]
        })
        
    # Write to file
    with open(filename, 'w') as f:
        json.dump(output_data, f, indent=2)
        
    print(f"Successfully exported {len(output_data['nodes'])} nodes and {len(output_data['edges'])} edges to '{filename}'")
    
    # Return string for preview
    return json.dumps(output_data, indent=2)

if __name__ == "__main__":
    MAP_SIZE = 100
    DENSITY = 300
    
    print("Generating city points...")
    points = generate_city_points(DENSITY, MAP_SIZE, MAP_SIZE, iterations=3)
    
    print("Building street graph...")
    street_graph = build_graph_from_voronoi(points, MAP_SIZE, MAP_SIZE)
    
    print("Exporting to JSON...")
    json_str = export_graph_to_json(street_graph, "city_data.json")
    
    # Preview the first few lines of JSON output
    print("\nJSON Preview:")
    print("\n".join(json_str.split("\n")[:20]))
    print("...")
