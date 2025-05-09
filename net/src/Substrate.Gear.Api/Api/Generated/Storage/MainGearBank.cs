#nullable disable
//------------------------------------------------------------------------------
// <auto-generated>
//     This code was generated by a tool.
//
//     Changes to this file may cause incorrect behavior and will be lost if
//     the code is regenerated.
// </auto-generated>
//------------------------------------------------------------------------------

using Substrate.NetApi;
using Substrate.NetApi.Model.Extrinsics;
using Substrate.NetApi.Model.Meta;
using Substrate.NetApi.Model.Types;
using Substrate.NetApi.Model.Types.Base;
using System.Collections.Generic;
using System.Threading;
using System.Threading.Tasks;


namespace Substrate.Gear.Api.Generated.Storage
{
    
    
    /// <summary>
    /// >> GearBankStorage
    /// </summary>
    public sealed class GearBankStorage
    {
        
        // Substrate client for the storage calls.
        private SubstrateClientExt _client;
        
        /// <summary>
        /// >> GearBankStorage Constructor
        /// </summary>
        public GearBankStorage(SubstrateClientExt client)
        {
            this._client = client;
            _client.StorageKeyDict.Add(new System.Tuple<string, string>("GearBank", "Bank"), new System.Tuple<Substrate.NetApi.Model.Meta.Storage.Hasher[], System.Type, System.Type>(new Substrate.NetApi.Model.Meta.Storage.Hasher[] {
                            Substrate.NetApi.Model.Meta.Storage.Hasher.Identity}, typeof(Substrate.Gear.Api.Generated.Model.sp_core.crypto.AccountId32), typeof(Substrate.Gear.Api.Generated.Model.pallet_gear_bank.pallet.BankAccount)));
            _client.StorageKeyDict.Add(new System.Tuple<string, string>("GearBank", "UnusedValue"), new System.Tuple<Substrate.NetApi.Model.Meta.Storage.Hasher[], System.Type, System.Type>(null, null, typeof(Substrate.NetApi.Model.Types.Primitive.U128)));
            _client.StorageKeyDict.Add(new System.Tuple<string, string>("GearBank", "OnFinalizeTransfers"), new System.Tuple<Substrate.NetApi.Model.Meta.Storage.Hasher[], System.Type, System.Type>(new Substrate.NetApi.Model.Meta.Storage.Hasher[] {
                            Substrate.NetApi.Model.Meta.Storage.Hasher.Identity}, typeof(Substrate.Gear.Api.Generated.Model.sp_core.crypto.AccountId32), typeof(Substrate.NetApi.Model.Types.Primitive.U128)));
            _client.StorageKeyDict.Add(new System.Tuple<string, string>("GearBank", "OnFinalizeValue"), new System.Tuple<Substrate.NetApi.Model.Meta.Storage.Hasher[], System.Type, System.Type>(null, null, typeof(Substrate.NetApi.Model.Types.Primitive.U128)));
        }
        
        /// <summary>
        /// >> BankParams
        /// </summary>
        public static string BankParams(Substrate.Gear.Api.Generated.Model.sp_core.crypto.AccountId32 key)
        {
            return RequestGenerator.GetStorage("GearBank", "Bank", Substrate.NetApi.Model.Meta.Storage.Type.Map, new Substrate.NetApi.Model.Meta.Storage.Hasher[] {
                        Substrate.NetApi.Model.Meta.Storage.Hasher.Identity}, new Substrate.NetApi.Model.Types.IType[] {
                        key});
        }
        
        /// <summary>
        /// >> BankDefault
        /// Default value as hex string
        /// </summary>
        public static string BankDefault()
        {
            return "0x00";
        }
        
        /// <summary>
        /// >> Bank
        /// </summary>
        public async Task<Substrate.Gear.Api.Generated.Model.pallet_gear_bank.pallet.BankAccount> Bank(Substrate.Gear.Api.Generated.Model.sp_core.crypto.AccountId32 key, string blockhash, CancellationToken token)
        {
            string parameters = GearBankStorage.BankParams(key);
            var result = await _client.GetStorageAsync<Substrate.Gear.Api.Generated.Model.pallet_gear_bank.pallet.BankAccount>(parameters, blockhash, token);
            return result;
        }
        
        /// <summary>
        /// >> UnusedValueParams
        /// </summary>
        public static string UnusedValueParams()
        {
            return RequestGenerator.GetStorage("GearBank", "UnusedValue", Substrate.NetApi.Model.Meta.Storage.Type.Plain);
        }
        
        /// <summary>
        /// >> UnusedValueDefault
        /// Default value as hex string
        /// </summary>
        public static string UnusedValueDefault()
        {
            return "0x00000000000000000000000000000000";
        }
        
        /// <summary>
        /// >> UnusedValue
        /// </summary>
        public async Task<Substrate.NetApi.Model.Types.Primitive.U128> UnusedValue(string blockhash, CancellationToken token)
        {
            string parameters = GearBankStorage.UnusedValueParams();
            var result = await _client.GetStorageAsync<Substrate.NetApi.Model.Types.Primitive.U128>(parameters, blockhash, token);
            return result;
        }
        
        /// <summary>
        /// >> OnFinalizeTransfersParams
        /// </summary>
        public static string OnFinalizeTransfersParams(Substrate.Gear.Api.Generated.Model.sp_core.crypto.AccountId32 key)
        {
            return RequestGenerator.GetStorage("GearBank", "OnFinalizeTransfers", Substrate.NetApi.Model.Meta.Storage.Type.Map, new Substrate.NetApi.Model.Meta.Storage.Hasher[] {
                        Substrate.NetApi.Model.Meta.Storage.Hasher.Identity}, new Substrate.NetApi.Model.Types.IType[] {
                        key});
        }
        
        /// <summary>
        /// >> OnFinalizeTransfersDefault
        /// Default value as hex string
        /// </summary>
        public static string OnFinalizeTransfersDefault()
        {
            return "0x00";
        }
        
        /// <summary>
        /// >> OnFinalizeTransfers
        /// </summary>
        public async Task<Substrate.NetApi.Model.Types.Primitive.U128> OnFinalizeTransfers(Substrate.Gear.Api.Generated.Model.sp_core.crypto.AccountId32 key, string blockhash, CancellationToken token)
        {
            string parameters = GearBankStorage.OnFinalizeTransfersParams(key);
            var result = await _client.GetStorageAsync<Substrate.NetApi.Model.Types.Primitive.U128>(parameters, blockhash, token);
            return result;
        }
        
        /// <summary>
        /// >> OnFinalizeValueParams
        /// </summary>
        public static string OnFinalizeValueParams()
        {
            return RequestGenerator.GetStorage("GearBank", "OnFinalizeValue", Substrate.NetApi.Model.Meta.Storage.Type.Plain);
        }
        
        /// <summary>
        /// >> OnFinalizeValueDefault
        /// Default value as hex string
        /// </summary>
        public static string OnFinalizeValueDefault()
        {
            return "0x00000000000000000000000000000000";
        }
        
        /// <summary>
        /// >> OnFinalizeValue
        /// </summary>
        public async Task<Substrate.NetApi.Model.Types.Primitive.U128> OnFinalizeValue(string blockhash, CancellationToken token)
        {
            string parameters = GearBankStorage.OnFinalizeValueParams();
            var result = await _client.GetStorageAsync<Substrate.NetApi.Model.Types.Primitive.U128>(parameters, blockhash, token);
            return result;
        }
    }
    
    /// <summary>
    /// >> GearBankCalls
    /// </summary>
    public sealed class GearBankCalls
    {
    }
    
    /// <summary>
    /// >> GearBankConstants
    /// </summary>
    public sealed class GearBankConstants
    {
        
        /// <summary>
        /// >> BankAddress
        ///  Bank account address, that will keep all reserved funds.
        /// </summary>
        public Substrate.Gear.Api.Generated.Model.sp_core.crypto.AccountId32 BankAddress()
        {
            var result = new Substrate.Gear.Api.Generated.Model.sp_core.crypto.AccountId32();
            result.Create("0x6765617262616E6B6765617262616E6B6765617262616E6B6765617262616E6B");
            return result;
        }
        
        /// <summary>
        /// >> GasMultiplier
        ///  Gas price converter.
        /// </summary>
        public Substrate.Gear.Api.Generated.Model.gear_common.EnumGasMultiplier GasMultiplier()
        {
            var result = new Substrate.Gear.Api.Generated.Model.gear_common.EnumGasMultiplier();
            result.Create("0x0006000000000000000000000000000000");
            return result;
        }
    }
    
    /// <summary>
    /// >> GearBankErrors
    /// </summary>
    public enum GearBankErrors
    {
        
        /// <summary>
        /// >> InsufficientBalance
        /// Insufficient user balance.
        /// </summary>
        InsufficientBalance,
        
        /// <summary>
        /// >> InsufficientGasBalance
        /// Insufficient user's bank account gas balance.
        /// </summary>
        InsufficientGasBalance,
        
        /// <summary>
        /// >> InsufficientValueBalance
        /// Insufficient user's bank account gas balance.
        /// </summary>
        InsufficientValueBalance,
        
        /// <summary>
        /// >> InsufficientBankBalance
        /// Insufficient bank account balance.
        /// **Must be unreachable in Gear main protocol.**
        /// </summary>
        InsufficientBankBalance,
        
        /// <summary>
        /// >> InsufficientDeposit
        /// Deposit of funds that will not keep bank account alive.
        /// **Must be unreachable in Gear main protocol.**
        /// </summary>
        InsufficientDeposit,
        
        /// <summary>
        /// >> Overflow
        /// Overflow during funds transfer.
        /// **Must be unreachable in Gear main protocol.**
        /// </summary>
        Overflow,
    }
}
