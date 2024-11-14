namespace Sails.ClientGenerator;

public record struct GeneratorConfig(
    string ServiceName,
    string Namespace
)
{
    public readonly string ToJsonString()
        => $"{{ \"service_name\": \"{this.ServiceName}\", \"namespace\": \"{this.Namespace}\" }}";
}
