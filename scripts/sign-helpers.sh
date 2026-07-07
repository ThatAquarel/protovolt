#!/usr/bin/env bash
# OpenSSL Ed25519 firmware sign/verify.
# PRIVATE_KEY / PUBLIC_KEY: PEM content in the env var, or a readable file path.

set -euo pipefail

_key_is_file() {
  [[ -n "${1:-}" && -f "$1" ]]
}

# Resolve a key path when recipes run from protov-hal/ but paths are repo-root relative.
_resolve_key_path() {
  local key="$1"
  if _key_is_file "$key"; then
    printf '%s' "$(cd "$(dirname "$key")" && pwd)/$(basename "$key")"
    return 0
  fi
  if [[ "$key" != /* && -f "../$key" ]]; then
    printf '%s' "$(cd "$(dirname "../$key")" && pwd)/$(basename "$key")"
    return 0
  fi
  return 1
}

_key_ref() {
  local key="$1"
  local resolved
  if resolved="$(_resolve_key_path "$key" 2>/dev/null)"; then
    printf '%s' "$resolved"
  else
    printf '%s' "$key"
  fi
}

hash_firmware() {
  local firmware_bin="$1"
  local digest_path="$2"
  mkdir -p "$(dirname "$digest_path")"
  openssl dgst -sha512 -binary "$firmware_bin" > "$digest_path"
}

sign_digest() {
  local digest_path="$1"
  local signature_path="$2"
  local key
  key="$(_key_ref "${PRIVATE_KEY:?PRIVATE_KEY is required}")"
  mkdir -p "$(dirname "$signature_path")"
  if _key_is_file "$key"; then
    openssl pkeyutl -sign -inkey "$key" -rawin -in "$digest_path" -out "$signature_path"
  else
    openssl pkeyutl -sign -inkey <(printf '%s' "$key") -rawin -in "$digest_path" -out "$signature_path"
  fi
}

verify_firmware() {
  local firmware_bin="$1"
  local signature_path="$2"
  local pub
  pub="$(_key_ref "${PUBLIC_KEY:?PUBLIC_KEY is required}")"
  local base="${firmware_bin%.bin}"
  local digest_path="${3:-}"
  local cleanup=0
  if [[ -z "$digest_path" ]]; then
    if [[ -f "${base}.hash.bin" ]]; then
      digest_path="${base}.hash.bin"
    elif [[ -f "${base}_hash.bin" ]]; then
      digest_path="${base}_hash.bin"
    else
      digest_path="$(mktemp)"
      cleanup=1
      hash_firmware "$firmware_bin" "$digest_path"
    fi
  fi
  if _key_is_file "$pub"; then
    openssl pkeyutl -verify -pubin -inkey "$pub" -rawin -in "$digest_path" -sigfile "$signature_path"
  else
    openssl pkeyutl -verify -pubin -inkey <(printf '%s' "$pub") -rawin -in "$digest_path" -sigfile "$signature_path"
  fi
  if (( cleanup )); then
    rm -f "$digest_path"
  fi
}

sign_firmware_bin() {
  local firmware_bin="$1"
  local base="${firmware_bin%.bin}"
  local digest_path="${base}.hash.bin"
  local signature_path="${base}.sign.bin"
  hash_firmware "$firmware_bin" "$digest_path"
  sign_digest "$digest_path" "$signature_path"
  verify_firmware "$firmware_bin" "$signature_path"
  echo "signed ${firmware_bin} -> ${signature_path}"
}
