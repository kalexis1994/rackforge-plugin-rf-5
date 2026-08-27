param(
    [string]$OutputRoot = ""
)

$ErrorActionPreference = "Stop"
$rfRepoRoot = Split-Path -Parent $PSScriptRoot
if ([string]::IsNullOrWhiteSpace($OutputRoot)) {
    $OutputRoot = Join-Path $rfRepoRoot "artifacts\portable-reference-comparison"
}
if (Test-Path -LiteralPath $OutputRoot) {
    throw "Refusing to overwrite an existing comparison: $OutputRoot"
}

$rfPortable = Join-Path $OutputRoot "portable"
$rfHostRatePrecise = Join-Path $OutputRoot "host-rate-precise"
$rfTwoTimes = Join-Path $OutputRoot "candidate-2x-fast-math"
$rfFourTimesFastMath = Join-Path $OutputRoot "reference-4x-fast-math"
$rfReference = Join-Path $OutputRoot "reference-4x"
$rfReport = Join-Path $OutputRoot "analysis"
$rfHostRateReport = Join-Path $OutputRoot "analysis-host-rate-precise"
$rfTwoTimesReport = Join-Path $OutputRoot "analysis-2x-fast-math"
$rfFastMathReport = Join-Path $OutputRoot "analysis-4x-fast-math"
New-Item -ItemType Directory -Path $rfPortable -Force | Out-Null
New-Item -ItemType Directory -Path $rfHostRatePrecise -Force | Out-Null
New-Item -ItemType Directory -Path $rfTwoTimes -Force | Out-Null
New-Item -ItemType Directory -Path $rfFourTimesFastMath -Force | Out-Null
New-Item -ItemType Directory -Path $rfReference -Force | Out-Null
New-Item -ItemType Directory -Path $rfReport -Force | Out-Null
New-Item -ItemType Directory -Path $rfHostRateReport -Force | Out-Null
New-Item -ItemType Directory -Path $rfTwoTimesReport -Force | Out-Null
New-Item -ItemType Directory -Path $rfFastMathReport -Force | Out-Null

Push-Location $rfRepoRoot
try {
    cargo run --locked --release -p rf-5-audition --features portable-realtime --bin rf-5-audition -- $rfPortable
    if ($LASTEXITCODE -ne 0) { throw "Portable RF-5 render failed" }

    cargo run --locked --release -p rf-5-audition --no-default-features --features host-rate --bin rf-5-audition -- $rfHostRatePrecise
    if ($LASTEXITCODE -ne 0) { throw "Host-rate precise-math RF-5 render failed" }

    cargo run --locked --release -p rf-5-audition --no-default-features --features two-times,fast-math --bin rf-5-audition -- $rfTwoTimes
    if ($LASTEXITCODE -ne 0) { throw "Two-times fast-math RF-5 render failed" }

    cargo run --locked --release -p rf-5-audition --no-default-features --features fast-math --bin rf-5-audition -- $rfFourTimesFastMath
    if ($LASTEXITCODE -ne 0) { throw "Four-times fast-math RF-5 render failed" }

    cargo run --locked --release -p rf-5-audition --no-default-features --bin rf-5-audition -- $rfReference
    if ($LASTEXITCODE -ne 0) { throw "Four-times RF-5 reference render failed" }

    cargo run --locked --release -p rf-5-audition --features portable-realtime --bin rf-5-compare -- $rfPortable $rfReference $rfReport "portable host-rate + bounded math and low-resonance solver" "four-times + precise math"
    if ($LASTEXITCODE -ne 0) { throw "RF-5 profile comparison failed" }

    cargo run --locked --release -p rf-5-audition --features portable-realtime --bin rf-5-compare -- $rfHostRatePrecise $rfReference $rfHostRateReport "host-rate + precise math" "four-times + precise math"
    if ($LASTEXITCODE -ne 0) { throw "RF-5 host-rate attribution comparison failed" }

    cargo run --locked --release -p rf-5-audition --features portable-realtime --bin rf-5-compare -- $rfTwoTimes $rfReference $rfTwoTimesReport "two-times + fast math candidate" "four-times + precise math"
    if ($LASTEXITCODE -ne 0) { throw "RF-5 two-times candidate comparison failed" }

    cargo run --locked --release -p rf-5-audition --features portable-realtime --bin rf-5-compare -- $rfFourTimesFastMath $rfReference $rfFastMathReport "four-times + fast math" "four-times + precise math"
    if ($LASTEXITCODE -ne 0) { throw "RF-5 fast-math attribution comparison failed" }
}
finally {
    Pop-Location
}

Write-Output "RF5_PROFILE_COMPARISON_READY path=$rfReport"
