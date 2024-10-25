using EnsureThat;
using Microsoft.Extensions.DependencyInjection;
using Sails.Remoting.Abstractions;
using Sails.Remoting.Options;

namespace Sails.Remoting.DependencyInjection;

public static class IServiceCollectionExtensions
{
    public static IServiceCollection AddRemotingViaSubstrateClient(
        this IServiceCollection services,
        RemotingViaSubstrateClientOptions options)
    {
        EnsureArg.IsNotNull(services, nameof(services));
        EnsureArg.IsNotNull(options, nameof(options));

        services.AddTransient<IRemotingProvider>(
            _ => new RemotingProvider(
                signingAccount => new RemotingViaSubstrateClient(options, signingAccount)));

        return services;
    }
}
