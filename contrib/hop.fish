# Hop shell wrapper for Fish
# Source with: hop init --shell fish | source
#
# The binary uses exit code 42 to signal "cd to stdout".

function hop
  set -l out (command hop $argv)
  set -l rc $status

  switch $rc
    case 42  # CD signal from binary
      cd "$out"
    case 0
      test -n "$out" && printf '%s\n' $out
    case '*'
      test -n "$out" && printf '%s\n' $out
      return $rc
  end
end

function h
  hop $argv
end
