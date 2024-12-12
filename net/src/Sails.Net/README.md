Sails.Net
=========

A library providing and implementing low-level abstractions allowing to communicate with programs
written using [Sails](https://github.com/gear-tech/sails?tab=readme-ov-file#sails-) framework.

Although the library can be used independently, it offers a better experience when paired with
[Sails.ClientGenerator](https://www.nuget.org/packages/Sails.ClientGenerator).

### Usage

Install the library, register `IRemotingProvider` implementation with `IServiceCollection`: 
```csharp
var serviceCollection = new ServiceCollection();
serviceCollection.AddRemotingViaNodeClient(
    new NodeClientOptions
    {
        GearNodeUri = new Uri("wss://testnet.vara.network"),
    });
```
Inject `IRemotingProvider` into your services and use it to communicate with Sails programs.
Discover how it can be achieved via exploring [tests](https://github.com/gear-tech/sails/tree/master/net/tests/Sails.Remoting.Tests).
