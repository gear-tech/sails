using System.Text;
using Microsoft.CodeAnalysis;
using Microsoft.CodeAnalysis.Text;

namespace Sails.ClientGenerator.Tests;

internal sealed class InMemoryAdditionalText : AdditionalText
{
    private readonly SourceText content;

    public InMemoryAdditionalText(string path, string content)
    {
        this.Path = path;
        this.content = SourceText.From(content, Encoding.UTF8);
    }

    public override string Path { get; }

    public override SourceText GetText(CancellationToken cancellationToken = default) => this.content;
}
