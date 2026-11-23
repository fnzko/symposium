# Source this file from bash or zsh: `. ./env.sh`
# It points HOME at a throwaway folder so symposium and the agents write
# nowhere in your real home. Toolchains (rustup, cargo cache, asdf, mise)
# keep using your real folders.

if [ -z "${SYMPOSIUM_DEMO_REAL_HOME:-}" ]; then
  export SYMPOSIUM_DEMO_REAL_HOME="$HOME"
fi
_real="$SYMPOSIUM_DEMO_REAL_HOME"

export DEMO="${DEMO:-$_real/symposium-demo}"
export RUSTUP_HOME="${RUSTUP_HOME:-$_real/.rustup}"
export CARGO_HOME="${CARGO_HOME:-$_real/.cargo}"
[ -d "$_real/.asdf" ] && export ASDF_DATA_DIR="${ASDF_DATA_DIR:-$_real/.asdf}"
[ -d "$_real/.local/share/mise" ] && export MISE_DATA_DIR="${MISE_DATA_DIR:-$_real/.local/share/mise}"
[ -d "$_real/.nvm" ] && export NVM_DIR="${NVM_DIR:-$_real/.nvm}"

unset SYMPOSIUM_HOME CLAUDE_CONFIG_DIR CODEX_HOME COPILOT_HOME GOOSE_PATH_ROOT XDG_CONFIG_HOME

[ -d "$_real/.config/mise" ] && export MISE_CONFIG_DIR="${MISE_CONFIG_DIR:-$_real/.config/mise}"

export HOME="$DEMO/home"
mkdir -p "$HOME" "$DEMO/bin"
[ -f "$_real/.tool-versions" ] && ln -sf "$_real/.tool-versions" "$HOME/.tool-versions"
case ":$PATH:" in
  *":$DEMO/bin:"*) ;;
  *) export PATH="$DEMO/bin:$PATH" ;;
esac
unset _real

echo "demo shell: HOME=$HOME"
echo "symposium on PATH: $(command -v symposium || echo 'not installed yet')"
