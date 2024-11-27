using System;
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
        Action<NodeClientOptions> configure)
    {
        EnsureArg.IsNotNull(services, nameof(services));
        EnsureArg.IsNotNull(configure, nameof(configure));

        var serviceCollection = services.Configure(configure);
        services.AddSingleton<INodeClientProvider, NodeClientProvider>();

        services.AddTransient<IRemotingProvider>(
            serviceProvicer => new RemotingProvider(
                signingAccount => new RemotingViaNodeClient(
                    serviceProvicer.GetRequiredService<INodeClientProvider>(),
                    signingAccount),
                () => new RemotingListenerViaNodeClient(serviceProvicer.GetRequiredService<INodeClientProvider>())
            )
        );

        return services;
    }
}
