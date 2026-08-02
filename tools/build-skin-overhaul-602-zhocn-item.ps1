[CmdletBinding()]
param(
    [Parameter(Mandatory = $true)]
    [string]$Unpacked602ItemDirectory,

    [Parameter(Mandatory = $true)]
    [string]$WitchyBndPath,

    [Parameter(Mandatory = $true)]
    [string]$BuildWorkspace,

    [string]$NameMapPath = (Join-Path $PSScriptRoot "skin-overhaul-602-zhocn-names.json")
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

function Get-FullPath([string]$Path, [string]$Label) {
    if (-not (Test-Path -LiteralPath $Path)) {
        throw "$Label does not exist: $Path"
    }

    return (Resolve-Path -LiteralPath $Path).Path
}

$sourceDirectory = Get-FullPath $Unpacked602ItemDirectory "Unpacked 602 item directory"
$witchy = Get-FullPath $WitchyBndPath "WitchyBND"
$mapFile = Get-FullPath $NameMapPath "Skin name map"

if (Test-Path -LiteralPath $BuildWorkspace) {
    throw "The build workspace already exists. Supply a new empty directory to avoid overwriting an earlier artifact: $BuildWorkspace"
}

$nameMapDocument = Get-Content -LiteralPath $mapFile -Raw -Encoding UTF8 | ConvertFrom-Json
$nameMap = @($nameMapDocument.entries.PSObject.Properties)
if ($nameMap.Count -ne 76) {
    throw "The name map must contain 76 entries; it contains $($nameMap.Count)."
}

New-Item -ItemType Directory -Path $BuildWorkspace | Out-Null
$copiedDirectory = Join-Path $BuildWorkspace "item_dlc01-msgbnd-dcx"
Copy-Item -LiteralPath $sourceDirectory -Destination $copiedDirectory -Recurse

$goodsNameXml = Join-Path $copiedDirectory "GoodsName.fmg.xml"
if (-not (Test-Path -LiteralPath $goodsNameXml)) {
    throw "GoodsName.fmg.xml is missing from the unpacked 602 item directory: $goodsNameXml"
}

[xml]$document = Get-Content -LiteralPath $goodsNameXml -Raw -Encoding UTF8
$entriesNode = $document.SelectSingleNode("/fmg/entries")
if ($null -eq $entriesNode) {
    throw "GoodsName.fmg.xml is missing the /fmg/entries node."
}

$existing = @{}
foreach ($node in $entriesNode.SelectNodes("text")) {
    $existing[[int]$node.GetAttribute("id")] = $node
}

foreach ($property in $nameMap) {
    $id = [int]$property.Name
    $translatedName = [string]$property.Value
    if ([string]::IsNullOrWhiteSpace($translatedName) -or $translatedName.Contains("?")) {
        throw "The translated name for clothing ID $id is invalid: $translatedName"
    }

    if ($existing.ContainsKey($id)) {
        $existing[$id].InnerText = $translatedName
    }
    else {
        $node = $document.CreateElement("text")
        [void]$node.SetAttribute("id", [string]$id)
        $node.InnerText = $translatedName
        [void]$entriesNode.AppendChild($node)
        $existing[$id] = $node
    }
}

$sortedNodes = @($entriesNode.SelectNodes("text") | Sort-Object { [int]$_.GetAttribute("id") })
foreach ($node in $sortedNodes) {
    [void]$entriesNode.RemoveChild($node)
}
foreach ($node in $sortedNodes) {
    [void]$entriesNode.AppendChild($node)
}

$writerSettings = [System.Xml.XmlWriterSettings]::new()
$writerSettings.Encoding = [System.Text.UTF8Encoding]::new($false)
$writerSettings.Indent = $true
$writerSettings.NewLineChars = "`n"
$writer = [System.Xml.XmlWriter]::Create($goodsNameXml, $writerSettings)
try {
    $document.Save($writer)
}
finally {
    $writer.Dispose()
}

& $witchy -s -r $goodsNameXml
if ($LASTEXITCODE -ne 0) {
    throw "WitchyBND failed to repack GoodsName.fmg; exit code: $LASTEXITCODE"
}

& $witchy -s -r $copiedDirectory
if ($LASTEXITCODE -ne 0) {
    throw "WitchyBND failed to repack item_dlc01.msgbnd.dcx; exit code: $LASTEXITCODE"
}

$outputItem = Join-Path $BuildWorkspace "item_dlc01.msgbnd.dcx"
if (-not (Test-Path -LiteralPath $outputItem)) {
    throw "The repacked item_dlc01.msgbnd.dcx was not found: $outputItem"
}

Write-Output "[OK] Created $outputItem"
Write-Output "[OK] Merged $($nameMap.Count) Skin Overhaul clothing names."
