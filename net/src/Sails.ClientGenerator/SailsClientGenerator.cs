using Microsoft.CodeAnalysis;
using Microsoft.CodeAnalysis.CSharp.Syntax;

namespace Sails.ClientGenerator;

[Generator(LanguageNames.CSharp)]
public partial class SailsClientGenerator : IIncrementalGenerator
{
    public const string SailsClientAttributeFullName = "Sails.ClientGenerator.SailsClientAttribute";

    public void Initialize(IncrementalGeneratorInitializationContext context)
    {
        context.RegisterPostInitializationOutput(ctx =>
        {
            ctx.AddSource("SailsClientAttribute.g.cs", """
namespace Sails.ClientGenerator
{
    [System.AttributeUsage(System.AttributeTargets.Class)]
    public class SailsClientAttribute : System.Attribute
    {
    }
}
""");
        });

        var source = context.SyntaxProvider.ForAttributeWithMetadataName(
            SailsClientAttributeFullName,
            predicate: static (node, token) => node is ClassDeclarationSyntax,
            transform: static (context, token) => context);

        context.RegisterSourceOutput(source, Generate);
    }

    private static void Generate(SourceProductionContext context, GeneratorAttributeSyntaxContext source)
    {

    }
}
