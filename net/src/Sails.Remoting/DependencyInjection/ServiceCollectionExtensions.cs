using EnsureThat;
using Microsoft.Extensions.DependencyInjection;
using Sails.Remoting.Abstractions.Core;
using Sails.Remoting.Core;
using Sails.Remoting.Options;

namespace Sails.Remoting.DependencyInjection;

public static class ServiceCollectionExtensions
{
    public static IServiceCollection AddRemotingViaNodeClient(
        this IServiceCollection services,
        NodeClientOptions options)
    {
        EnsureArg.IsNotNull(services, nameof(services));
        EnsureArg.IsNotNull(options, nameof(options));

        services.AddSingleton<INodeClientProvider>(_ => new NodeClientProvider(options));

        services.AddTransient<IRemotingProvider>(
            serviceProvicer => new RemotingProvider(
                signingAccount => new RemotingViaNodeClient(
                    serviceProvicer.GetRequiredService<INodeClientProvider>(),
                    signingAccount)));

        return services;
    }
}
