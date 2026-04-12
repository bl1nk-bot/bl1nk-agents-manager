#!/usr/bin/env bash
# ==============================================================================
# bumpversion.sh - อัปเดต version ใน Cargo.toml และสร้าง git tag
#
# Usage:
#   ./scripts/bumpversion.sh patch    # 0.1.0 -> 0.1.1
#   ./scripts/bumpversion.sh minor    # 0.1.0 -> 0.2.0
#   ./scripts/bumpversion.sh major    # 0.1.0 -> 1.0.0
#   ./scripts/bumpversion.sh 1.2.3    # ตั้งค่า version โดยตรง
#
# Options:
#   --dry-run    แสดงสิ่งที่จะทำแต่ไม่แก้ไขไฟล์
#   --no-tag     ไม่สร้าง git tag
#   --no-commit  ไม่ commit การเปลี่ยนแปลง
# ==============================================================================

set -euo pipefail

# สีสำหรับ terminal
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Defaults
DRY_RUN=false
NO_TAG=false
NO_COMMIT=false
CARGO_TOML="Cargo.toml"

# ฟังก์ชันแสดงวิธีใช้
usage() {
    cat <<EOF
Usage: $(basename "$0") [OPTIONS] <version|patch|minor|major>

Arguments:
  patch, minor, major   bumps version according to semver
  <version>             sets version explicitly (e.g. 1.2.3)

Options:
  --dry-run             แสดงสิ่งที่จะทำแต่ไม่แก้ไขไฟล์
  --no-tag              ไม่สร้าง git tag
  --no-commit           ไม่ commit การเปลี่ยนแปลง
  -h, --help            แสดงวิธีใช้

Examples:
  $(basename "$0") patch
  $(basename "$0") minor
  $(basename "$0") 2.0.0
  $(basename "$0") --dry-run minor
  $(basename "$0") --no-tag --no-commit patch
EOF
    exit 0
}

# ฟังก์ชัน logging
log_info()  { echo -e "${BLUE}[INFO]${NC}  $*"; }
log_ok()    { echo -e "${GREEN}[OK]${NC}    $*"; }
log_warn()  { echo -e "${YELLOW}[WARN]${NC}  $*"; }
log_error() { echo -e "${RED}[ERROR]${NC} $*"; }

# Parse arguments
BUMP_TYPE=""
while [[ $# -gt 0 ]]; do
    case "$1" in
        --dry-run)  DRY_RUN=true;  shift ;;
        --no-tag)   NO_TAG=true;   shift ;;
        --no-commit) NO_COMMIT=true; shift ;;
        -h|--help)  usage ;;
        -*)         log_error "Unknown option: $1"; usage ;;
        *)          BUMP_TYPE="$1"; shift ;;
    esac
done

if [[ -z "$BUMP_TYPE" ]]; then
    log_error "ต้องระบุ version type (patch, minor, major) หรือ version number"
    usage
fi

# ตรวจสอบว่า Cargo.toml มีอยู่จริง
if [[ ! -f "$CARGO_TOML" ]]; then
    log_error "ไม่พบ $CARGO_TOML ใน directory ปัจจุบัน"
    exit 1
fi

# อ่าน version ปัจจุบัน
CURRENT_VERSION=$(grep -E '^version\s*=' "$CARGO_TOML" | head -1 | sed -E 's/^version\s*=\s*"([^"]+)"/\1/')

if [[ -z "$CURRENT_VERSION" ]]; then
    log_error "ไม่สามารถอ่าน version จาก $CARGO_TOML ได้"
    exit 1
fi

log_info "Current version: $CURRENT_VERSION"

# แยกส่วน version
IFS='.' read -r MAJOR MINOR PATCH <<< "$CURRENT_VERSION"

# คำนวณ version ใหม่
case "$BUMP_TYPE" in
    patch)
        NEW_VERSION="$MAJOR.$MINOR.$((PATCH + 1))"
        ;;
    minor)
        NEW_VERSION="$MAJOR.$((MINOR + 1)).0"
        ;;
    major)
        NEW_VERSION="$((MAJOR + 1)).0.0"
        ;;
    *)
        # ตรวจสอบว่าเป็น valid semver หรือไม่
        if [[ "$BUMP_TYPE" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
            NEW_VERSION="$BUMP_TYPE"
        else
            log_error "Invalid version: $BUMP_TYPE (ต้องเป็น patch, minor, major, หรือ semver เช่น 1.2.3)"
            exit 1
        fi
        ;;
esac

log_info "New version:     $NEW_VERSION"

if [[ "$DRY_RUN" == true ]]; then
    log_warn "--dry-run mode: จะไม่แก้ไขไฟล์จริง"
    echo ""
    echo "สิ่งที่ทำ:"
    echo "  1. แก้ไข version ใน $CARGO_TOML: $CURRENT_VERSION -> $NEW_VERSION"
    if [[ "$NO_COMMIT" != true ]]; then
        echo "  2. Commit: chore(release): bump version to $NEW_VERSION"
    fi
    if [[ "$NO_TAG" != true ]]; then
        echo "  3. Git tag: v$NEW_VERSION"
    fi
    echo ""
    log_ok "Dry run finished"
    exit 0
fi

# อัปเดต version ใน Cargo.toml
sed -i "s/^version = \"$CURRENT_VERSION\"/version = \"$NEW_VERSION\"/" "$CARGO_TOML"
log_ok "Updated $CARGO_TOML: $CURRENT_VERSION -> $NEW_VERSION"

# ตรวจสอบว่ามีไฟล์อื่นที่ต้องแก้ไหม (เช่น Cargo.lock)
if [[ -f "Cargo.lock" ]]; then
    log_info "Running cargo update เพื่ออัปเดต Cargo.lock..."
    cargo update --quiet
    log_ok "Cargo.lock updated"
fi

# Commit การเปลี่ยนแปลง
if [[ "$NO_COMMIT" != true ]]; then
    git add "$CARGO_TOML"
    if [[ -f "Cargo.lock" ]]; then
        git add Cargo.lock
    fi

    git commit -m "chore(release): bump version to $NEW_VERSION

Bump version from $CURRENT_VERSION to $NEW_VERSION

Commit: $(git rev-parse --short HEAD)
Date:   $(date -u +'%Y-%m-%d %H:%M:%S UTC')"

    log_ok "Committed: chore(release): bump version to $NEW_VERSION"
else
    log_warn "--no-commit: ไม่ได้ commit"
fi

# สร้าง git tag
if [[ "$NO_TAG" != true ]]; then
    TAG="v$NEW_VERSION"

    # ตรวจสอบว่า tag มีอยู่แล้วหรือไม่
    if git tag -l | grep -q "^$TAG$"; then
        log_warn "Tag $TAG มีอยู่แล้ว ข้ามการสร้าง"
    else
        git tag -a "$TAG" -m "Release $TAG

Version: $NEW_VERSION
Date:    $(date -u +'%Y-%m-%d %H:%M:%S UTC')

Changes from $CURRENT_VERSION to $NEW_VERSION"

        log_ok "Created tag: $TAG"
    fi

    echo ""
    log_info "Push ไปยัง remote ด้วยคำสั่ง:"
    echo "  git push origin main"
    echo "  git push origin $TAG"
else
    log_warn "--no-tag: ไม่ได้สร้าง git tag"
fi

echo ""
log_ok "Version bump complete: $CURRENT_VERSION -> $NEW_VERSION"
