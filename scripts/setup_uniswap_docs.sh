#!/bin/bash

# Create docs directory structure
mkdir -p docs/uniswap/{v2,v3}/{contracts,documentation,guides}

# Clone Uniswap V2 repositories
echo "Fetching Uniswap V2 documentation and contracts..."
git clone https://github.com/Uniswap/v2-core.git temp/v2-core
git clone https://github.com/Uniswap/v2-periphery.git temp/v2-periphery
git clone https://github.com/Uniswap/docs.git temp/docs

# Process V2 documentation
echo "Processing V2 documentation..."
cp -r temp/v2-core/contracts/* docs/uniswap/v2/contracts/
cp -r temp/v2-periphery/contracts/* docs/uniswap/v2/contracts/
cp -r temp/docs/docs/V2/* docs/uniswap/v2/documentation/

# Clone Uniswap V3 repositories
echo "Fetching Uniswap V3 documentation and contracts..."
git clone https://github.com/Uniswap/v3-core.git temp/v3-core
git clone https://github.com/Uniswap/v3-periphery.git temp/v3-periphery

# Process V3 documentation
echo "Processing V3 documentation..."
cp -r temp/v3-core/contracts/* docs/uniswap/v3/contracts/
cp -r temp/v3-periphery/contracts/* docs/uniswap/v3/contracts/
cp -r temp/docs/docs/V3/* docs/uniswap/v3/documentation/

# Clean up temporary files
echo "Cleaning up..."
rm -rf temp

echo "Documentation setup complete!"
