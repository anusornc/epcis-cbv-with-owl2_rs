#!/usr/bin/env python3

import requests
import json

def test_real_data():
    """Test if we're getting real EPCIS data"""
    
    # Test SPARQL query
    url = "http://localhost:8082/api/v1/sparql/query"
    payload = {
        "query": "SELECT * WHERE { ?s ?p ?o } LIMIT 10",
        "default_graph_uri": None,
        "named_graph_uri": None
    }
    
    try:
        response = requests.post(url, json=payload)
        data = response.json()
        
        print("🔍 SPARQL Query Results:")
        print(f"Status: {data.get('status')}")
        print(f"Query type: {data.get('query_type')}")
        print(f"Execution time: {data.get('execution_time_ms')}ms")
        
        bindings = data.get('results', {}).get('bindings', [])
        print(f"Number of results: {len(bindings)}")
        
        print("\n📊 Sample results:")
        for i, binding in enumerate(bindings[:5]):
            s = binding.get('s', {}).get('value', 'N/A')
            p = binding.get('p', {}).get('value', 'N/A')
            o = binding.get('o', {}).get('value', 'N/A')
            print(f"  {i+1}. {s} -> {p} -> {o}")
        
        # Check if we have real EPCIS data
        has_epcis = any(
            'epcis' in binding.get('s', {}).get('value', '').lower() or
            'epcis' in binding.get('p', {}).get('value', '').lower() or
            'epcis' in binding.get('o', {}).get('value', '').lower() or
            'cbv' in binding.get('s', {}).get('value', '').lower() or
            'cbv' in binding.get('p', {}).get('value', '').lower() or
            'cbv' in binding.get('o', {}).get('value', '').lower()
            for binding in bindings
        )
        
        has_real_uris = any(
            not binding.get('s', {}).get('value', '').startswith('ex:resource') and
            not binding.get('p', {}).get('value', '').startswith('ex:') and
            not binding.get('o', {}).get('value', '').startswith('value')
            for binding in bindings
        )
        
        print(f"\n🎯 Data Analysis:")
        print(f"  Contains EPCIS terms: {'✅' if has_epcis else '❌'}")
        print(f"  Contains real URIs: {'✅' if has_real_uris else '❌'}")
        
        if has_epcis and has_real_uris:
            print("🎉 SUCCESS: Getting real EPCIS data!")
        elif has_real_uris:
            print("⚠️  Getting some real data but no EPCIS terms")
        else:
            print("❌ Still getting mock/sample data")
            
        return has_epcis and has_real_uris
        
    except Exception as e:
        print(f"❌ Error: {e}")
        return False

if __name__ == "__main__":
    test_real_data()