#!/bin/bash
# Example: Create Dify Plugins using the Creator Framework

CREATOR_DIR="/home/user/skills/huynguyen03dev/dify-plugin-creator"

echo "=== Dify Plugin Creator Examples ==="
echo ""

# Example 1: Document Processor
echo "Example 1: Creating Document Processor Plugin"
python $CREATOR_DIR/scripts/init_dify_plugin.py \
  --name "doc-processor" \
  --author "your-name" \
  --type "document_processor" \
  --output-dir "/tmp/dify-plugins"

echo ""

# Example 2: API Wrapper
echo "Example 2: Creating API Wrapper Plugin"
python $CREATOR_DIR/scripts/init_dify_plugin.py \
  --name "weather-api" \
  --author "your-name" \
  --type "api_wrapper" \
  --output-dir "/tmp/dify-plugins"

echo ""

# Example 3: Contact Manager
echo "Example 3: Creating Contact Manager Plugin"
python $CREATOR_DIR/scripts/init_dify_plugin.py \
  --name "contact-hub" \
  --author "your-name" \
  --type "contact_manager" \
  --output-dir "/tmp/dify-plugins"

echo ""
echo "Plugins created in /tmp/dify-plugins"
