using System.Collections.Immutable;
using System.Text;
using Microsoft.CodeAnalysis;
using Microsoft.CodeAnalysis.CSharp;
using Microsoft.CodeAnalysis.Text;

namespace Sails.ClientGenerator;

[Generator(LanguageNames.CSharp)]
public partial class SailsClientGenerator : IIncrementalGenerator
{
    public void Initialize(IncrementalGeneratorInitializationContext context)
    {
        var source = context.AdditionalTextsProvider
            .Where(static file => file.Path.EndsWith(".idl"));

        var compilationAndFiles = context.CompilationProvider
            .Select((c, _) => c.AssemblyName)
            .Combine(source.Collect());

        context.RegisterSourceOutput(compilationAndFiles, AddSource);
    }

    private static void AddSource(
        SourceProductionContext context,
        (string? AssemblyName, ImmutableArray<AdditionalText> Right) tuple)
    {
        var assemblyName = tuple.AssemblyName!;
        foreach (var source in tuple.Right)
        {
            // TODO: add relative directory as namespace part
            var parts = new List<string>();
            parts.Insert(0, assemblyName);
            var name = FirstUpper(Path.GetFileNameWithoutExtension(source.Path));
            parts.Add(name);
            var ns = string.Join(".", parts);
            var code = GenerateCode(source.GetText()!.ToString(), new GeneratorConfig(name, ns));

            context.AddSource($"{name}.g.cs", SourceText.From(code, encoding: Encoding.UTF8));
        }
    }

    private static unsafe string GenerateCode(string source, GeneratorConfig config)
    {
        var handle = NativeMethods.LoadNativeLibrary();
        try
        {
            var idlBytes = Encoding.UTF8.GetBytes(source);
            var configBytes = Encoding.UTF8.GetBytes(config.ToString());

            fixed (byte* pIdl = idlBytes)
            {
                fixed (byte* pConfig = configBytes)
                {
                    var cstr = NativeMethods.generate_dotnet_client(pIdl, idlBytes.Length, pConfig, configBytes.Length);
                    try
                    {
                        var str = new string((sbyte*)cstr);
                        return FormatCode(str);
                    }
                    finally
                    {
                        NativeMethods.free_c_string(cstr);
                    }
                }
            }
        }
        finally
        {
            NativeMethods.FreeNativeLibrary(handle);
        }
    }

    private static string FormatCode(string code, CancellationToken cancellationToken = default)
        => CSharpSyntaxTree.ParseText(code, cancellationToken: cancellationToken)
            .GetRoot(cancellationToken)
            .NormalizeWhitespace()
            .SyntaxTree
            .GetText(cancellationToken)
            .ToString();

    private static string FirstUpper(string text)
    {
        if (text.Length == 0)
        {
            return text;
        }
        Span<char> res = stackalloc char[text.Length];
        text.AsSpan().CopyTo(res);
        var c = res[0];
        if (char.IsLetter(c) && char.IsLower(c))
        {
            res[0] = char.ToUpperInvariant(c);
        }
        return res.ToString();
    }
}
