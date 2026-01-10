# Hop shell wrapper for PowerShell
# Source with: hop init --shell powershell | Invoke-Expression
#
# The binary uses exit code 42 to signal "cd to stdout".

function hop {
    param([Parameter(ValueFromRemainingArguments=$true)][string[]]$Args)

    $exe = (Get-Command hop -CommandType Application -ErrorAction SilentlyContinue).Path
    if (-not $exe) { $exe = 'hop' }

    $out = & $exe @Args 2>&1
    $rc = $LASTEXITCODE

    switch ($rc) {
        42 { Set-Location ($out -join "`n").Trim() }
        0  { if ($out) { Write-Output $out } }
        default { if ($out) { Write-Output $out }; return $rc }
    }
}

function h { hop @Args }
