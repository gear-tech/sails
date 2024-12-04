param (
    [string]$directory = ".",
    [string]$fileType = "*.*",
    [string]$header
)

# Get all files in the directory with the specified file type
Get-ChildItem -Path $directory -Filter $fileType -File -Recurse | ForEach-Object {
    # Read the file content as a single string
    $content = Get-Content $_.FullName -Raw

    # Add header
    $newContent = $header + [System.Environment]::NewLine + $content

    # Write the updated content back to the file, preserving the original line endings
    [System.IO.File]::WriteAllText($_.FullName, $newContent)
}
