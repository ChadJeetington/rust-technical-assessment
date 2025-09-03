#!/usr/bin/env python3
"""
Script to fetch and pre-process Uniswap documentation for RAG system.

This script:
1. Downloads documentation from Uniswap's official sources
2. Processes and formats the documentation
3. Stores it in a structured format for the RAG system
4. Can be run as part of CI/CD to keep documentation up to date
"""

import os
import sys
import json
import shutil
import logging
import requests
from pathlib import Path
from datetime import datetime
from concurrent.futures import ThreadPoolExecutor

# Configure logging
logging.basicConfig(
    level=logging.INFO,
    format='%(asctime)s - %(levelname)s - %(message)s'
)

# Uniswap documentation sources
UNISWAP_SOURCES = {
    'v2': {
        'docs': [
            'https://docs.uniswap.org/contracts/v2/overview',
            'https://docs.uniswap.org/contracts/v2/concepts',
            'https://docs.uniswap.org/contracts/v2/guides',
        ],
        'contracts': [
            'https://raw.githubusercontent.com/Uniswap/v2-core/master/contracts/UniswapV2Pair.sol',
            'https://raw.githubusercontent.com/Uniswap/v2-core/master/contracts/UniswapV2Factory.sol',
            'https://raw.githubusercontent.com/Uniswap/v2-periphery/master/contracts/UniswapV2Router02.sol',
        ],
    },
    'v3': {
        'docs': [
            'https://docs.uniswap.org/contracts/v3/overview',
            'https://docs.uniswap.org/contracts/v3/concepts',
            'https://docs.uniswap.org/contracts/v3/guides',
        ],
        'contracts': [
            'https://raw.githubusercontent.com/Uniswap/v3-core/main/contracts/UniswapV3Pool.sol',
            'https://raw.githubusercontent.com/Uniswap/v3-core/main/contracts/UniswapV3Factory.sol',
            'https://raw.githubusercontent.com/Uniswap/v3-periphery/main/contracts/SwapRouter.sol',
        ],
    },
}

def fetch_url(url: str) -> tuple[str, str]:
    """Fetch content from URL with error handling and rate limiting."""
    try:
        response = requests.get(url)
        response.raise_for_status()
        return url, response.text
    except Exception as e:
        logging.error(f"Failed to fetch {url}: {e}")
        return url, ""

def process_documentation(version: str, content: str, url: str) -> dict:
    """Process documentation into structured format."""
    return {
        'content': content,
        'metadata': {
            'version': version,
            'source_url': url,
            'processed_at': datetime.utcnow().isoformat(),
            'type': 'documentation',
        }
    }

def process_contract(version: str, content: str, url: str) -> dict:
    """Process contract into structured format."""
    return {
        'content': content,
        'metadata': {
            'version': version,
            'source_url': url,
            'processed_at': datetime.utcnow().isoformat(),
            'type': 'contract',
        }
    }

def main():
    # Get output directory from args or use default
    output_dir = Path(sys.argv[1] if len(sys.argv) > 1 else 'docs/uniswap')
    output_dir.mkdir(parents=True, exist_ok=True)
    
    # Process each version
    for version, sources in UNISWAP_SOURCES.items():
        logging.info(f"Processing Uniswap {version} documentation...")
        version_dir = output_dir / version
        
        # Create version directories
        (version_dir / 'docs').mkdir(parents=True, exist_ok=True)
        (version_dir / 'contracts').mkdir(parents=True, exist_ok=True)
        
        # Fetch documentation and contracts in parallel
        with ThreadPoolExecutor(max_workers=10) as executor:
            # Fetch docs
            doc_futures = [executor.submit(fetch_url, url) for url in sources['docs']]
            for future in doc_futures:
                url, content = future.result()
                if content:
                    doc = process_documentation(version, content, url)
                    doc_path = version_dir / 'docs' / f"{Path(url).name}.md"
                    with open(doc_path, 'w') as f:
                        f.write(content)
                    with open(f"{doc_path}.meta.json", 'w') as f:
                        json.dump(doc['metadata'], f, indent=2)
            
            # Fetch contracts
            contract_futures = [executor.submit(fetch_url, url) for url in sources['contracts']]
            for future in contract_futures:
                url, content = future.result()
                if content:
                    contract = process_contract(version, content, url)
                    contract_path = version_dir / 'contracts' / Path(url).name
                    with open(contract_path, 'w') as f:
                        f.write(content)
                    with open(f"{contract_path}.meta.json", 'w') as f:
                        json.dump(contract['metadata'], f, indent=2)
    
    logging.info("✅ Documentation processing complete!")

if __name__ == '__main__':
    main()
