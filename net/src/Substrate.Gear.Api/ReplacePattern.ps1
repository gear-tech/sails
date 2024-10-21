param (
    [string]$directory = ".",
    [string]$fileType = "*.*",
    [string]$searchPattern,
    [string]$replacePattern,
    [switch]$caseInsensitive
)

# Ensure both patterns are provided
if (-not $searchPattern -or -not $replacePattern) {
    Write-Error "Both searchPattern and replacePattern must be provided."
    exit 1
}

# Get all files in the directory with the specified file type
Get-ChildItem -Path $directory -Filter $fileType -File -Recurse | ForEach-Object {
    # Read the file content
    $content = Get-Content $_.FullName

    # Replace the pattern, case-sensitive if the switch is set
    if ($caseInsensitive) {
        $newContent = $content -replace $searchPattern, $replacePattern
    }
    else {
        $newContent = $content -creplace $searchPattern, $replacePattern
    }

    # Write the updated content back to the file
    Set-Content -Path $_.FullName -Value $newContent
}
