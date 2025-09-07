#!/usr/bin/env python3
"""
Test script to verify the visualization fix in working_version.html
"""

import requests
import json
import time

def test_sparql_api():
    """Test the SPARQL API endpoint"""
    print("🔍 Testing SPARQL API...")
    
    url = "http://localhost:8082/api/v1/sparql/query"
    payload = {
        "query": "SELECT * WHERE { ?s ?p ?o } LIMIT 15",
        "default_graph_uri": None,
        "named_graph_uri": None
    }
    
    try:
        response = requests.post(url, json=payload)
        if response.status_code == 200:
            data = response.json()
            print(f"✅ SPARQL API working - returned {len(data['results']['bindings'])} results")
            return data
        else:
            print(f"❌ SPARQL API failed: {response.status_code}")
            return None
    except Exception as e:
        print(f"❌ SPARQL API error: {e}")
        return None

def test_static_files():
    """Test static file serving"""
    print("📄 Testing static file serving...")
    
    url = "http://localhost:8082/static/working_version.html"
    try:
        response = requests.get(url)
        if response.status_code == 200:
            print("✅ Static files serving correctly")
            return True
        else:
            print(f"❌ Static files failed: {response.status_code}")
            return False
    except Exception as e:
        print(f"❌ Static files error: {e}")
        return False

def convert_sparql_to_graph(bindings):
    """Convert SPARQL bindings to graph format (same as JavaScript function)"""
    nodes = {}
    links = []
    
    for binding in bindings:
        subject = binding.get('s', {}).get('value')
        predicate = binding.get('p', {}).get('value')
        object = binding.get('o', {}).get('value')
        
        if subject and predicate and object:
            # Add subject node
            if subject not in nodes:
                nodes[subject] = {
                    'id': subject,
                    'name': get_node_name(subject),
                    'type': get_node_type(subject),
                    'group': get_node_group(subject)
                }
            
            # Add object node
            if object not in nodes:
                nodes[object] = {
                    'id': object,
                    'name': get_node_name(object),
                    'type': get_node_type(object),
                    'group': get_node_group(object)
                }
            
            # Add link
            links.append({
                'source': subject,
                'target': object,
                'type': get_edge_type(predicate),
                'label': get_edge_label(predicate)
            })
    
    return {
        'nodes': list(nodes.values()),
        'links': links
    }

def get_node_name(uri):
    """Get node name from URI"""
    if uri.startswith('ex:'):
        return uri.replace('ex:', '').replace('resource', 'Resource ')
    if '#' in uri:
        return uri.split('#')[-1]
    if '/' in uri:
        return uri.split('/')[-1]
    return uri[:30] + '...' if len(uri) > 30 else uri

def get_node_type(uri):
    """Get node type from URI"""
    uri_lower = uri.lower()
    if 'product' in uri_lower:
        return 'product'
    elif 'location' in uri_lower:
        return 'location'
    elif 'event' in uri_lower:
        return 'event'
    elif 'business' in uri_lower:
        return 'business'
    return 'resource'

def get_node_group(uri):
    """Get node group from URI"""
    node_type = get_node_type(uri)
    groups = {
        'product': 1,
        'location': 2,
        'event': 3,
        'business': 4
    }
    return groups.get(node_type, 0)

def get_edge_type(predicate):
    """Get edge type from predicate"""
    predicate_lower = predicate.lower()
    if 'type' in predicate_lower:
        return 'type'
    elif 'has' in predicate_lower:
        return 'has'
    return 'relationship'

def get_edge_label(predicate):
    """Get edge label from predicate"""
    if '#' in predicate:
        return predicate.split('#')[-1]
    return predicate[:20] + '...' if len(predicate) > 20 else predicate

def test_graph_conversion():
    """Test the SPARQL to graph conversion"""
    print("🔄 Testing SPARQL to graph conversion...")
    
    sparql_data = test_sparql_api()
    if not sparql_data:
        return False
    
    bindings = sparql_data['results']['bindings']
    print(f"📊 Converting {len(bindings)} SPARQL bindings to graph...")
    
    graph_data = convert_sparql_to_graph(bindings)
    
    print(f"✅ Conversion successful:")
    print(f"   - Nodes: {len(graph_data['nodes'])}")
    print(f"   - Links: {len(graph_data['links'])}")
    
    # Show sample nodes
    if graph_data['nodes']:
        print(f"   Sample nodes:")
        for i, node in enumerate(graph_data['nodes'][:3]):
            print(f"     {i+1}. {node['name']} ({node['type']})")
    
    # Show sample links
    if graph_data['links']:
        print(f"   Sample links:")
        for i, link in enumerate(graph_data['links'][:3]):
            print(f"     {i+1}. {link['source']} -> {link['target']} ({link['label']})")
    
    return True

def main():
    """Main test function"""
    print("🚀 Testing visualization fix...")
    print("=" * 50)
    
    # Test all components
    static_ok = test_static_files()
    sparql_ok = test_sparql_api()
    conversion_ok = test_graph_conversion()
    
    print("=" * 50)
    print("📋 Test Results:")
    print(f"   Static Files: {'✅' if static_ok else '❌'}")
    print(f"   SPARQL API: {'✅' if sparql_ok else '❌'}")
    print(f"   Graph Conversion: {'✅' if conversion_ok else '❌'}")
    
    if all([static_ok, sparql_ok, conversion_ok]):
        print("🎉 All tests passed! The visualization should work correctly.")
        print("📝 Visit: http://localhost:8082/static/working_version.html")
    else:
        print("❌ Some tests failed. Check the output above for details.")
    
    return all([static_ok, sparql_ok, conversion_ok])

if __name__ == "__main__":
    main()