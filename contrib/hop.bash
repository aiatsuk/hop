# Hop shell wrapper for Bash and Zsh
# Source with: eval "$(hop init --shell bash)"
#
# The binary uses exit code 42 to signal "cd to stdout".

hop() {
  local out rc
  out="$(command hop "$@")"
  rc=$?

  case $rc in
    42)  # CD signal from binary
      cd "$out" || return $?
      ;;
    0)
      [ -n "$out" ] && printf '%s\n' "$out"
      ;;
    *)
      [ -n "$out" ] && printf '%s\n' "$out"
      return $rc
      ;;
  esac
}

h() { hop "$@"; }
