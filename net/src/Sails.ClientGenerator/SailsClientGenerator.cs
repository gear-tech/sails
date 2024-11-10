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
        var source = context.AdditionalTextsProvider.Where(static file => file.Path.EndsWith(".idl"));

        context.RegisterSourceOutput(source, Generate);
    }

    private static unsafe void Generate(SourceProductionContext context, AdditionalText source)
    {
        var handle = NativeMethods.LoadNativeLibrary();
        try
        {
            var idl = source.GetText()!.ToString();
            var idlBytes = Encoding.UTF8.GetBytes(idl);

            var name = Path.GetFileNameWithoutExtension(source.Path);
            var nameBytes = Encoding.UTF8.GetBytes(name);

            fixed (byte* pIdl = idlBytes)
            {
                fixed (byte* pName = nameBytes)
                {
                    var cstr = NativeMethods.generate_dotnet_client(pIdl, idlBytes.Length, pName, nameBytes.Length);
                    try
                    {
                        var str = new string((sbyte*)cstr);
                        var formatted = FormatCode(str);
                        context.AddSource($"{name}.g.cs", SourceText.From(formatted, encoding: Encoding.UTF8));
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

    public static string FormatCode(string code, CancellationToken cancelToken = default)
        => CSharpSyntaxTree.ParseText(code, cancellationToken: cancelToken)
            .GetRoot(cancelToken)
            .NormalizeWhitespace()
            .SyntaxTree
            .GetText(cancelToken)
            .ToString();
}
