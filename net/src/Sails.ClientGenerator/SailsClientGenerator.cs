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
            .Where(static file => file.Path.EndsWith(".idl"))
            .Select(static (text, cancellationToken) =>
                GenerateCode(Path.GetFileNameWithoutExtension(text.Path), text.GetText(cancellationToken)!.ToString())
            );

        context.RegisterSourceOutput(source, AddSource);
    }

    private static unsafe (string Name, string Code) GenerateCode(string name, string source)
    {
        var handle = NativeMethods.LoadNativeLibrary();
        try
        {
            var idlBytes = Encoding.UTF8.GetBytes(source);
            var nameBytes = Encoding.UTF8.GetBytes(name);

            fixed (byte* pIdl = idlBytes)
            {
                fixed (byte* pName = nameBytes)
                {
                    var cstr = NativeMethods.generate_dotnet_client(pIdl, idlBytes.Length, pName, nameBytes.Length);
                    try
                    {
                        var str = new string((sbyte*)cstr);
                        return (name, FormatCode(str));
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

    private static void AddSource(SourceProductionContext context, (string Name, string Code) source)
    {
        context.AddSource($"{source.Name}.g.cs", SourceText.From(source.Code, encoding: Encoding.UTF8));
    }

    public static string FormatCode(string code, CancellationToken cancelToken = default)
        => CSharpSyntaxTree.ParseText(code, cancellationToken: cancelToken)
            .GetRoot(cancelToken)
            .NormalizeWhitespace()
            .SyntaxTree
            .GetText(cancelToken)
            .ToString();
}
