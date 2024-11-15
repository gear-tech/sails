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
        (string? AssemblyName, ImmutableArray<AdditionalText> Texts) tuple)
    {
        if (tuple.AssemblyName is null)
        {
            // Do not generated code without Assembly in Compilation
            return;
        }
        var assemblyName = tuple.AssemblyName!;
        foreach (var source in tuple.Texts)
        {
            var text = source.GetText();
            if (text is null)
            {
                continue;
            }
            // TODO: add relative directory as namespace part
            var parts = new List<string>();
            parts.Insert(0, assemblyName);
            var name = FirstUpper(Path.GetFileNameWithoutExtension(source.Path));
            parts.Add(name);
            var ns = string.Join(".", parts);
            var code = FormatCode(Generator.GenerateCode(text.ToString(), new GeneratorConfig(name, ns)));

            context.AddSource($"{name}.g.cs", SourceText.From(code, encoding: Encoding.UTF8));
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
