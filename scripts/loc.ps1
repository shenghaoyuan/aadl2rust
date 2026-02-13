# Absolute path to cloc (DO NOT rely on PATH)
$CLOC = "D:\tools\cloc\cloc.exe"

if (-not (Test-Path $CLOC)) {
    Write-Error "cloc executable not found at $CLOC"
    exit 1
}

Write-Host "===== Code Line Statistics (cloc, code-only) ====="

function Get-ClocCode {
    param (
        [string[]]$Paths
    )

    $json = & $CLOC @Paths --quiet --json | ConvertFrom-Json

    $code = 0
    foreach ($prop in $json.PSObject.Properties) {
        if ($prop.Name -ne "header" -and $prop.Name -ne "SUM") {
            $code += $prop.Value.code
        }
    }
    return $code
}

# 1. AST declaration
$ast = Get-ClocCode @(
    "src/ast.rs"
    "src/aadl_ast2rust_code/intermediate_ast.rs"
)
Write-Host ("[AST declaration]         {0}" -f $ast)

# 2. Parsing
$parsing = Get-ClocCode @(
    "src/aadl.pest",
    "src/transform.rs",
    "src/transform_annex.rs"
)
Write-Host ("[Parsing]                 {0}" -f $parsing)

# 3. Model-to-IR translation
$modelToIr = Get-ClocCode @(
    "src/aadl_ast2rust_code/converter.rs",
    "src/aadl_ast2rust_code/converter_annex.rs",
    "src/aadl_ast2rust_code/implementations",
    "src/aadl_ast2rust_code/types"
)
Write-Host ("[Model-to-IR translation] {0}" -f $modelToIr)

# 4. Rust code printer
$printer = Get-ClocCode @(
    "src/aadl_ast2rust_code/intermediate_print.rs"
)
Write-Host ("[Rust code printer]       {0}" -f $printer)

# Total
$total = $ast + $parsing + $modelToIr + $printer
Write-Host "-----------------------------------------------"
Write-Host ("[Total]                   {0}" -f $total)
