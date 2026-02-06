#!/bin/bash

# Script to clear Inboxed app data while preserving AI models
# Use this to reset the app without re-downloading the AI models

set -e

echo "🧹 Clearing Inboxed App Data (Keeping AI Models)"
echo "================================================"
echo ""

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Confirmation
echo -e "${YELLOW}⚠️  This will delete:${NC}"
echo "  - Email database and cache"
echo "  - OAuth tokens"
echo "  - All settings"
echo ""
echo -e "${GREEN}✓ AI models will be PRESERVED${NC}"
echo ""
read -p "Are you sure you want to continue? (y/N): " -n 1 -r
echo
if [[ ! $REPLY =~ ^[Yy]$ ]]; then
    echo "Cancelled."
    exit 1
fi

APP_DATA_DIR="$HOME/Library/Application Support/com.inboxed.inboxed"

# Backup models if they exist
MODELS_BACKUP=""
if [ -d "$APP_DATA_DIR/models" ]; then
    echo -e "${YELLOW}Backing up AI models...${NC}"
    MODELS_BACKUP=$(mktemp -d)
    cp -R "$APP_DATA_DIR/models" "$MODELS_BACKUP/"
    echo -e "${GREEN}✓ Models backed up to: $MODELS_BACKUP${NC}"
fi

# Clear main app data directory
if [ -d "$APP_DATA_DIR" ]; then
    echo -e "${YELLOW}Removing app data directory...${NC}"
    rm -rf "$APP_DATA_DIR"
    echo -e "${GREEN}✓ Removed app data directory${NC}"
else
    echo -e "${YELLOW}App data directory not found, skipping${NC}"
fi

# Restore models if we backed them up
if [ -n "$MODELS_BACKUP" ] && [ -d "$MODELS_BACKUP/models" ]; then
    echo -e "${YELLOW}Restoring AI models...${NC}"
    mkdir -p "$APP_DATA_DIR"
    cp -R "$MODELS_BACKUP/models" "$APP_DATA_DIR/"
    rm -rf "$MODELS_BACKUP"
    echo -e "${GREEN}✓ AI models restored${NC}"
fi

# Clear dev mode tokens
DEV_TOKENS_DIR="$HOME/.inboxed"
if [ -d "$DEV_TOKENS_DIR" ]; then
    echo -e "${YELLOW}Removing dev tokens...${NC}"
    rm -rf "$DEV_TOKENS_DIR"
    echo -e "${GREEN}✓ Removed dev tokens${NC}"
else
    echo -e "${YELLOW}Dev tokens not found, skipping${NC}"
fi

# Clear production keychain entries (macOS)
if [[ "$OSTYPE" == "darwin"* ]]; then
    echo -e "${YELLOW}Clearing keychain entries...${NC}"
    security delete-generic-password -s "com.inboxed.app" -a "gmail_access_token" 2>/dev/null && echo -e "${GREEN}✓ Removed access token${NC}" || echo -e "${YELLOW}No access token found${NC}"
    security delete-generic-password -s "com.inboxed.app" -a "gmail_refresh_token" 2>/dev/null && echo -e "${GREEN}✓ Removed refresh token${NC}" || echo -e "${YELLOW}No refresh token found${NC}"
    security delete-generic-password -s "com.inboxed.app" -a "gmail_token_expiry" 2>/dev/null && echo -e "${GREEN}✓ Removed token expiry${NC}" || echo -e "${YELLOW}No token expiry found${NC}"
fi

echo ""
echo -e "${GREEN}✅ Complete! App data cleared while preserving AI models.${NC}"
echo -e "${YELLOW}Next time you open the app, you'll need to sign in again.${NC}"
echo -e "${GREEN}Your AI models are ready to use immediately.${NC}"
