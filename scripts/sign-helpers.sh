#!/usr/bin/env bash
# OpenSSL Ed25519 sign/verify helpers for ProtoV tooling.
#
# Firmware release signing:
#   PRIVATE_KEY / PUBLIC_KEY — PEM content or readable .pem path
#
# Factory identity attestation (factory-run / factory-build):
#   Identity: SERIAL_NUMBER, HARDWARE_REV, FACTORY_YEAR, FACTORY_MONTH, FACTORY_DAY
#   Signing (one of):
#     SIGNATURE — 128 hex digits (64-byte Ed25519 signature), or
#     HW_PRIVATE_KEY + HW_PUBLIC_KEY — PEM paths/content; SIGNATURE is derived automatically
#
# Canonical attestation message (must match protov_nvm::encode_attestation_message):
#   {serial},{hw_revision},{YYYY-MM-DD}
# Example: 550e8400,A.1,2026-06-27
# Signed with Ed25519 over the raw UTF-8 message bytes (no pre-hash).

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

# --- Factory attestation (protov_nvm::encode_attestation_message) ---

# Bash mirror of protov_nvm::encode_attestation_message(serial, hw_revision, (year, month, day)).
# Output: {serial},{hw_revision},{YYYY-MM-DD}
encode_attestation_message() {
  local serial="$1"
  local hw_revision="$2"
  local year="$3"
  local month="$4"
  local day="$5"
  printf '%s,%s,%04d-%02d-%02d' "$serial" "$hw_revision" "$year" "$month" "$day"
}

_normalize_signature_hex() {
  local hex="$1"
  hex="${hex#"#H"}"
  hex="${hex//[[:space:]]/}"
  if [[ ! "$hex" =~ ^[0-9a-fA-F]{128}$ ]]; then
    echo "factory: SIGNATURE must be 128 hex digits (64 bytes)" >&2
    return 1
  fi
  printf '%s' "$(printf '%s' "$hex" | tr '[:upper:]' '[:lower:]')"
}

_require_factory_identity_env() {
  local missing=0
  for key in SERIAL_NUMBER HARDWARE_REV FACTORY_YEAR FACTORY_MONTH FACTORY_DAY; do
    if [[ -z "${!key:-}" ]]; then
      echo "factory: $key is required" >&2
      missing=1
    fi
  done
  if (( missing != 0 )); then
    exit 1
  fi
}

_verify_attestation_signature() {
  local message_file="$1"
  local sig_file="$2"
  local pub
  pub="$(_key_ref "${HW_PUBLIC_KEY:?HW_PUBLIC_KEY is required}")"
  if _key_is_file "$pub"; then
    openssl pkeyutl -verify -pubin -inkey "$pub" -rawin -in "$message_file" -sigfile "$sig_file"
  else
    openssl pkeyutl -verify -pubin -inkey <(printf '%s' "$pub") -rawin -in "$message_file" -sigfile "$sig_file"
  fi
}

_sign_attestation_message() {
  local message_file="$1"
  local sig_file="$2"
  local priv
  priv="$(_key_ref "${HW_PRIVATE_KEY:?HW_PRIVATE_KEY is required}")"
  if _key_is_file "$priv"; then
    openssl pkeyutl -sign -inkey "$priv" -rawin -in "$message_file" -out "$sig_file"
  else
    openssl pkeyutl -sign -inkey <(printf '%s' "$priv") -rawin -in "$message_file" -out "$sig_file"
  fi
}

# Resolve/export SIGNATURE for factory-program builds.
# Call from just factory-run / factory-build after identity env vars are set.
resolve_factory_signature() {
  _require_factory_identity_env

  local message message_file sig_bin
  message="$(encode_attestation_message \
    "$SERIAL_NUMBER" "$HARDWARE_REV" \
    "$FACTORY_YEAR" "$FACTORY_MONTH" "$FACTORY_DAY")"
  message_file="$(mktemp)"
  sig_bin="$(mktemp)"
  trap 'rm -f "$message_file" "$sig_bin"' RETURN

  printf '%s' "$message" > "$message_file"

  if [[ -n "${SIGNATURE:-}" ]]; then
    SIGNATURE="$(_normalize_signature_hex "$SIGNATURE")"
    printf '%s' "$SIGNATURE" | xxd -r -p > "$sig_bin"
    if [[ -n "${HW_PUBLIC_KEY:-}" ]]; then
      _verify_attestation_signature "$message_file" "$sig_bin"
    fi
    export SIGNATURE
    echo "factory: using SIGNATURE for attestation message: $message" >&2
    return 0
  fi

  if [[ -z "${HW_PRIVATE_KEY:-}" || -z "${HW_PUBLIC_KEY:-}" ]]; then
    cat >&2 <<EOF
factory: provide either:
  SIGNATURE           precomputed 128-hex-digit Ed25519 signature, or
  HW_PRIVATE_KEY      HW attestation private key (.pem path or inline PEM)
  HW_PUBLIC_KEY       matching public key (.pem path or inline PEM)

Required identity fields:
  SERIAL_NUMBER HARDWARE_REV FACTORY_YEAR FACTORY_MONTH FACTORY_DAY

Attestation message (encode_attestation_message):
  {serial},{hw_revision},{YYYY-MM-DD}
Rust reference: protov_nvm::encode_attestation_message
EOF
    exit 1
  fi

  _sign_attestation_message "$message_file" "$sig_bin"
  _verify_attestation_signature "$message_file" "$sig_bin"
  SIGNATURE="$(xxd -p -c 256 "$sig_bin" | tr -d '\n')"
  export SIGNATURE
  echo "factory: signed attestation message: $message" >&2
}
