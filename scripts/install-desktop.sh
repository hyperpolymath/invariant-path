#!/usr/bin/env bash
# SPDX-License-Identifier: MPL-2.0
# Install Invariant Path desktop/start-menu shortcuts.

set -euo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
REPO_DIR="$(cd -- "${SCRIPT_DIR}/.." && pwd)"
DESKTOP_TEMPLATE="${REPO_DIR}/desktop/invariant-path.desktop"
APPS_DIR="${HOME}/.local/share/applications"
DESKTOP_DIR="${HOME}/Desktop"
SHORTCUTS_DIR="${DESKTOP_DIR}/Shortcuts"
DESKTOP_FILE_NAME="invariant-path.desktop"
VERIFY_SCRIPT="/var/mnt/eclipse/repos/.desktop-tools/verify-desktop-integrity.sh"

mkdir -p "${APPS_DIR}" "${SHORTCUTS_DIR}"

if [[ ! -f "${DESKTOP_TEMPLATE}" ]]; then
  echo "Desktop template missing: ${DESKTOP_TEMPLATE}" >&2
  exit 1
fi

install_desktop_file() {
  local destination="$1"
  local mode="$2"

  # `install` refuses to update read-only files; replace first when needed.
  if [[ -e "${destination}" && ! -w "${destination}" ]]; then
    rm -f "${destination}"
  fi

  install -m "${mode}" "${DESKTOP_TEMPLATE}" "${destination}"
}

# Start-menu entries do not need execute bits.
install_desktop_file "${APPS_DIR}/${DESKTOP_FILE_NAME}" 444

# KDE/Plasma desktop launchers should be executable to avoid trust prompts.
install_desktop_file "${SHORTCUTS_DIR}/${DESKTOP_FILE_NAME}" 555
install_desktop_file "${DESKTOP_DIR}/${DESKTOP_FILE_NAME}" 555

if command -v update-desktop-database >/dev/null 2>&1; then
  update-desktop-database "${APPS_DIR}" >/dev/null 2>&1 || true
fi

if [[ -x "${VERIFY_SCRIPT}" ]]; then
  "${VERIFY_SCRIPT}" --generate >/dev/null 2>&1 || true
fi

echo "Installed:"
echo "  ${APPS_DIR}/${DESKTOP_FILE_NAME}"
echo "  ${SHORTCUTS_DIR}/${DESKTOP_FILE_NAME}"
echo "  ${DESKTOP_DIR}/${DESKTOP_FILE_NAME}"
