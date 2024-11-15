namespace Sails.ClientGenerator.Native;

public record struct GeneratorConfig(
    string ServiceName,
    string Namespace
)
{
    public override string ToString()
        => $"{{ \"service_name\": \"{this.ServiceName}\", \"namespace\": \"{this.Namespace}\" }}";
}
