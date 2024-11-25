using FluentAssertions;
using Substrate.Gear.Client.NetApi.Model.Rpc;
using Substrate.Gear.Client.NetApi.Model.Types.Base;
using Substrate.NetApi.Model.Rpc;
using Substrate.NetApi.Model.Types.Base;
using Substrate.NetApi.Model.Types.Primitive;
using Xunit;

namespace Substrate.Gear.Client.Tests.Model.Rpc;

public sealed class HeaderExtensionsTests
{
    [Theory]
    [MemberData(nameof(Block_Hash_Is_Calculated_Correctly), MemberType = typeof(TheoryData))]
    public void Block_Hash_Is_Calculated_Correctly((string ExpectedBlockHash, Header BlockHeader) data)
    {
        var blockHash = data.BlockHeader.GetBlockHash();
        var expectedBlockHash = new Hash(data.ExpectedBlockHash);

        blockHash.Should().Match<Hash>((hash) => hash.IsEqualTo(expectedBlockHash));
    }

    private static class TheoryData
    {
        // Taken from Vara Network
        public static readonly TheoryData<(string ExpectedBlockHash, Header BlockHeader)> Block_Hash_Is_Calculated_Correctly =
            new(
                (
                    ExpectedBlockHash: "0x0A7CA5B5B7A7B4C186D9B9355B5864DB74DD71DEAB700DEDA11BC852EA710E4A",
                    new Header
                    {
                        ExtrinsicsRoot = new Hash("0xd5c7e252243071f25d8013aa60e87e1650b4f069983eeafcecec10c0a03619ae"),
                        Number = (U64)16940619,
                        ParentHash = new Hash("0x9060eda7f8699cfcfa69c26640473bf5a322274eb391d066ec758c8f990c1eab"),
                        StateRoot = new Hash("0x7db88586b63b1ef556c13bec7a42040ebacacb2a1a8ad8517824674acf4f0106"),
                        Digest = new Digest
                        {
                            Logs = [
                                "0x0642414245340213000000ef9e612200000000",
                                "0x054241424501015832548cc0447f21a474529d904a7681a0e2e67baf90ef6d9f208b8e5dd4a22940e5e5b99c4e8a62e6f3a0b543883b07503cdcb40fe2c0c40a24df45dcd44f8e"
                            ]
                        }
                    }
                ),
                (
                    ExpectedBlockHash: "0x79F23DDD3EF8B1F7A9BA7ABED94574B779B73CE4E9D955657ABF53C38551F8BF",
                    new Header
                    {
                        ExtrinsicsRoot = new Hash("0x246c1d21868a9f8476e74d06a7b2cc0077703862b1cbeca546e25de9f203a1c6"),
                        Number = (U64)16940905,
                        ParentHash = new Hash("0x302955194b49e227fa40b111718a3449b763323e778aa89ca96ae9eb8ded3f64"),
                        StateRoot = new Hash("0xadac86b1731b2ebfb50a0c75f13ccfc884e21cab010cf511d2149836a61e9d6f"),
                        Digest = new Digest
                        {
                            Logs = [
                                "0x0642414245b501013100000011a0612200000000e4653ae16d22ee4766b802cae76c58273d793e59c70ddb173c2a63b452ec4820435f5d32fc12fcdbba1bd2b6a3f3349fd0356bbb003bbc3da1009cc5879c0d0d88baee2605129bd5cfb0e050df2c38aeaa927c548028a352dca52b954f375607",
                                "0x05424142450101e8dd5abf4f1b1f600f1257110b51bdfdf6b7bf5462c618c0b22ad39e979eb32b25e70cbc7ca4d1d229f25b839c0e01c94270e7332d6b7b73bf9a86bc09262b8f"
                            ]
                        }
                    }
                )
            );
    }
}
