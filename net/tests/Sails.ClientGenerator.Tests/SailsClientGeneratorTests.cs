using System.Collections.Immutable;
using System.Runtime.CompilerServices;
using Microsoft.CodeAnalysis;
using Microsoft.CodeAnalysis.CSharp;

namespace Sails.ClientGenerator.Tests;

public class SailsClientGeneratorTests
{
    [ModuleInitializer]
    internal static void Init() => VerifySourceGenerators.Initialize();

    [Fact]
    public Task Generate_DemoIdl() => Verify("idl/demo.idl");

    private static Task Verify(params string[] fileNames)
    {
        // Create a Roslyn compilation for the syntax tree.
        var compilation = CSharpCompilation.Create(assemblyName: "Sails.ClientGenerator.Tests");

        var additionalFiles = fileNames
            .Select(x => (file: x, content: File.ReadAllText($"./{x}")))
            .Select(x => new InMemoryAdditionalText(x.file, x.content))
            .ToImmutableArray();

        // Create an instance of our SailsClientGenerator incremental source generator
        var generator = new SailsClientGenerator().AsSourceGenerator();

        // The GeneratorDriver is used to run our generator against a compilation
        GeneratorDriver driver = CSharpGeneratorDriver.Create([generator], additionalTexts: additionalFiles);

        // Run the source generator!
        driver = driver.RunGenerators(compilation);

        // Use verify to snapshot test the source generator output!
        return Verifier.Verify(driver).UseDirectory("Snapshots");
    }
}
