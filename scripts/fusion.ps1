<#
.SYNOPSIS
    Rebuild the disposable `fusion-cargo` integration branch in an isolated worktree
    by merging multiple feature branches on top of main, without touching active worktrees.

.EXAMPLE
    ./scripts/fusion.ps1 -Push
    ./scripts/fusion.ps1 -Check
    ./scripts/fusion.ps1 -Branches feature/settings,feature/layer -Push
#>
param(
    [string[]]$Branches = @("feature/settings", "feature/layer", "feature/entity"),
    [string]$Base = "origin/main",
    [string]$TargetBranch = "fusion-cargo",
    [string]$WorktreeDir = "",
    [switch]$Push,
    [switch]$Check,
    [switch]$SkipFetch
)

$ErrorActionPreference = "Stop"

function Invoke-Git {
    param([string[]]$GitArgs)
    & git @GitArgs
    if ($LASTEXITCODE -ne 0) {
        throw "git $($GitArgs -join ' ') failed with exit code $LASTEXITCODE"
    }
}

function Test-GitRef {
    param([string]$Ref)
    & git rev-parse --verify --quiet $Ref *> $null
    return ($LASTEXITCODE -eq 0)
}

function Resolve-BranchRef {
    param([string]$Branch)

    $hasLocal = Test-GitRef "refs/heads/$Branch"
    $hasOrigin = Test-GitRef "refs/remotes/origin/$Branch"

    if ($hasLocal -and $hasOrigin) {
        & git merge-base --is-ancestor "refs/remotes/origin/$Branch" "refs/heads/$Branch"
        if ($LASTEXITCODE -eq 0) {
            return $Branch
        }
        & git merge-base --is-ancestor "refs/heads/$Branch" "refs/remotes/origin/$Branch"
        if ($LASTEXITCODE -eq 0) {
            Write-Host "  [info] local '$Branch' is behind 'origin/$Branch'; using 'origin/$Branch'" -ForegroundColor Yellow
            return "origin/$Branch"
        }
        Write-Warning "Local '$Branch' and 'origin/$Branch' have diverged; using local '$Branch'."
        return $Branch
    }
    elseif ($hasLocal) {
        return $Branch
    }
    elseif ($hasOrigin) {
        return "origin/$Branch"
    }
    else {
        throw "Branch '$Branch' not found locally or on origin."
    }
}

$repoRoot = (& git rev-parse --show-toplevel).Trim()
if ($LASTEXITCODE -ne 0 -or -not $repoRoot) {
    throw "Not inside a git repository."
}

$commonDir = (& git rev-parse --git-common-dir).Trim()
if (-not [System.IO.Path]::IsPathRooted($commonDir)) {
    $commonDir = [System.IO.Path]::GetFullPath((Join-Path $repoRoot $commonDir))
}
$mainParent = Split-Path (Split-Path $commonDir -Parent) -Parent
if (-not $WorktreeDir) {
    $WorktreeDir = Join-Path $mainParent "wt-$TargetBranch"
}

Push-Location $repoRoot
try {
    if (-not $SkipFetch) {
        Write-Host "==> Fetching origin..." -ForegroundColor Cyan
        Invoke-Git @("fetch", "origin", "--prune")
    }

    if (-not (Test-GitRef $Base)) {
        if ($Base -eq "origin/main" -and (Test-GitRef "main")) {
            $Base = "main"
        } else {
            throw "Base ref '$Base' does not exist."
        }
    }

    if (-not (Test-Path $WorktreeDir)) {
        Write-Host "==> Creating integration worktree at $WorktreeDir..." -ForegroundColor Cyan
        if (Test-GitRef "refs/heads/$TargetBranch") {
            Invoke-Git @("worktree", "add", $WorktreeDir, $TargetBranch)
        } else {
            Invoke-Git @("worktree", "add", "-b", $TargetBranch, $WorktreeDir, $Base)
        }
    } else {
        $wtStatus = & git -C $WorktreeDir status --porcelain
        if ($wtStatus) {
            Write-Host "==> Cleaning dirty state in $WorktreeDir..." -ForegroundColor Yellow
            Invoke-Git @("-C", $WorktreeDir, "reset", "--hard", "HEAD")
            Invoke-Git @("-C", $WorktreeDir, "clean", "-fd")
        }
        $currentWtBranch = (& git -C $WorktreeDir rev-parse --abbrev-ref HEAD).Trim()
        if ($currentWtBranch -ne $TargetBranch) {
            if (Test-GitRef "refs/heads/$TargetBranch") {
                Invoke-Git @("-C", $WorktreeDir, "switch", $TargetBranch)
            } else {
                Invoke-Git @("-C", $WorktreeDir, "switch", "-c", $TargetBranch, $Base)
            }
        }
    }

    Write-Host "==> Resetting '$TargetBranch' to $Base..." -ForegroundColor Cyan
    Invoke-Git @("-C", $WorktreeDir, "reset", "--hard", $Base)

    foreach ($branch in $Branches) {
        $ref = Resolve-BranchRef $branch
        $sha = (& git rev-parse --short $ref).Trim()
        Write-Host "==> Merging $ref ($sha) into $TargetBranch..." -ForegroundColor Cyan
        & git -C $WorktreeDir merge --no-ff --no-edit $ref
        if ($LASTEXITCODE -ne 0) {
            Write-Error "Merge conflict while merging '$ref' into '$TargetBranch' inside $WorktreeDir. Resolve or abort inside $WorktreeDir; your feature branches were not modified."
            exit 1
        }
    }

    if ($Check) {
        Write-Host "==> Running cargo xtask check in $WorktreeDir..." -ForegroundColor Cyan
        Push-Location $WorktreeDir
        try {
            & cargo xtask check
            if ($LASTEXITCODE -ne 0) {
                exit $LASTEXITCODE
            }
        } finally {
            Pop-Location
        }
    }

    if ($Push) {
        Write-Host "==> Pushing $TargetBranch to origin..." -ForegroundColor Cyan
        Invoke-Git @("-C", $WorktreeDir, "push", "--force-with-lease", "origin", $TargetBranch)
        Write-Host "==> Pushed 'origin/$TargetBranch'. GitHub Actions CI will run cargo xtask check." -ForegroundColor Green
    }

    if (-not $Check -and -not $Push) {
        Write-Host ""
        Write-Host "Integration branch '$TargetBranch' built at: $WorktreeDir" -ForegroundColor Green
        Write-Host "Next steps:"
        Write-Host "  - Run CI in GitHub Actions: ./scripts/fusion.ps1 -Push"
        Write-Host "  - Run check locally:        ./scripts/fusion.ps1 -Check"
    }
} finally {
    Pop-Location
}
