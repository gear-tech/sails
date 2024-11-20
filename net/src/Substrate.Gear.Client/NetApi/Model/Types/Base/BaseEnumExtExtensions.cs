using System;
using System.Diagnostics.CodeAnalysis;
using EnsureThat;
using Substrate.NetApi.Model.Types;
using Substrate.NetApi.Model.Types.Base;

namespace Substrate.Gear.Client.NetApi.Model.Types.Base;

[SuppressMessage("Roslynator", "RCS0056:A line is too long", Justification = "Foreign code")]
public static class BaseEnumExtExtensions
{
    public static BaseEnumRust<TEnum> ToBaseEnumRust<TEnum, T0, T1, T2>(
        this BaseEnumExt<TEnum, T0, T1, T2> baseEnumExt)
        where TEnum : Enum where T0 : IType, new() where T1 : IType, new() where T2 : IType, new()
    {
        EnsureArg.IsNotNull(baseEnumExt, nameof(baseEnumExt));

        var baseEnumRust = new BaseEnumRust<TEnum>();
        baseEnumRust.AddTypeDecoder<T0>((TEnum)(object)0);
        baseEnumRust.AddTypeDecoder<T1>((TEnum)(object)1);
        baseEnumRust.AddTypeDecoder<T2>((TEnum)(object)2);

        var idx = 0;
        baseEnumRust.Decode(baseEnumExt.Bytes, ref idx);

        return baseEnumRust;
    }

    public static BaseEnumRust<TEnum> ToBaseEnumRust<TEnum, T0, T1, T2, T3, T4, T5>(
        this BaseEnumExt<TEnum, T0, T1, T2, T3, T4, T5> baseEnumExt)
        where TEnum : Enum where T0 : IType, new() where T1 : IType, new() where T2 : IType, new() where T3 : IType, new() where T4 : IType, new() where T5 : IType, new()
    {
        EnsureArg.IsNotNull(baseEnumExt, nameof(baseEnumExt));

        var baseEnumRust = new BaseEnumRust<TEnum>();
        baseEnumRust.AddTypeDecoder<T0>((TEnum)(object)0);
        baseEnumRust.AddTypeDecoder<T1>((TEnum)(object)1);
        baseEnumRust.AddTypeDecoder<T2>((TEnum)(object)2);
        baseEnumRust.AddTypeDecoder<T3>((TEnum)(object)3);
        baseEnumRust.AddTypeDecoder<T4>((TEnum)(object)4);
        baseEnumRust.AddTypeDecoder<T5>((TEnum)(object)5);

        var idx = 0;
        baseEnumRust.Decode(baseEnumExt.Bytes, ref idx);

        return baseEnumRust;
    }

    public static BaseEnumRust<TEnum> ToBaseEnumRust<TEnum, T0, T1, T2, T3, T4, T5, T6, T7, T8>(
        this BaseEnumExt<TEnum, T0, T1, T2, T3, T4, T5, T6, T7, T8> baseEnumExt)
        where TEnum : Enum where T0 : IType, new() where T1 : IType, new() where T2 : IType, new() where T3 : IType, new() where T4 : IType, new() where T5 : IType, new() where T6 : IType, new() where T7 : IType, new() where T8 : IType, new()
    {
        EnsureArg.IsNotNull(baseEnumExt, nameof(baseEnumExt));

        var baseEnumRust = new BaseEnumRust<TEnum>();
        baseEnumRust.AddTypeDecoder<T0>((TEnum)(object)0);
        baseEnumRust.AddTypeDecoder<T1>((TEnum)(object)1);
        baseEnumRust.AddTypeDecoder<T2>((TEnum)(object)2);
        baseEnumRust.AddTypeDecoder<T3>((TEnum)(object)3);
        baseEnumRust.AddTypeDecoder<T4>((TEnum)(object)4);
        baseEnumRust.AddTypeDecoder<T5>((TEnum)(object)5);
        baseEnumRust.AddTypeDecoder<T6>((TEnum)(object)6);
        baseEnumRust.AddTypeDecoder<T7>((TEnum)(object)7);
        baseEnumRust.AddTypeDecoder<T8>((TEnum)(object)8);

        var idx = 0;
        baseEnumRust.Decode(baseEnumExt.Bytes, ref idx);

        return baseEnumRust;
    }

    public static BaseEnumRust<TEnum> ToBaseEnumRust<TEnum, T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13>(
        this BaseEnumExt<TEnum, T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13> baseEnumExt)
        where TEnum : Enum where T0 : IType, new() where T1 : IType, new() where T2 : IType, new() where T3 : IType, new() where T4 : IType, new() where T5 : IType, new() where T6 : IType, new() where T7 : IType, new() where T8 : IType, new() where T9 : IType, new() where T10 : IType, new() where T11 : IType, new() where T12 : IType, new() where T13 : IType, new()
    {
        EnsureArg.IsNotNull(baseEnumExt, nameof(baseEnumExt));

        var baseEnumRust = new BaseEnumRust<TEnum>();
        baseEnumRust.AddTypeDecoder<T0>((TEnum)(object)0);
        baseEnumRust.AddTypeDecoder<T1>((TEnum)(object)1);
        baseEnumRust.AddTypeDecoder<T2>((TEnum)(object)2);
        baseEnumRust.AddTypeDecoder<T3>((TEnum)(object)3);
        baseEnumRust.AddTypeDecoder<T4>((TEnum)(object)4);
        baseEnumRust.AddTypeDecoder<T5>((TEnum)(object)5);
        baseEnumRust.AddTypeDecoder<T6>((TEnum)(object)6);
        baseEnumRust.AddTypeDecoder<T7>((TEnum)(object)7);
        baseEnumRust.AddTypeDecoder<T8>((TEnum)(object)8);
        baseEnumRust.AddTypeDecoder<T9>((TEnum)(object)9);
        baseEnumRust.AddTypeDecoder<T10>((TEnum)(object)10);
        baseEnumRust.AddTypeDecoder<T11>((TEnum)(object)11);
        baseEnumRust.AddTypeDecoder<T12>((TEnum)(object)12);
        baseEnumRust.AddTypeDecoder<T13>((TEnum)(object)13);

        var idx = 0;
        baseEnumRust.Decode(baseEnumExt.Bytes, ref idx);

        return baseEnumRust;
    }

    public static BaseEnumRust<TEnum> ToBaseEnumRust<TEnum, T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15, T16, T17, T18, T19, T20, T21, T22, T23, T24, T25, T26, T27, T28, T29, T30, T31, T32, T33, T34, T35, T36, T37, T38, T39, T40, T41, T42, T43, T44, T45, T46, T47, T48, T49, T50, T51, T52, T53, T54, T55, T56, T57, T58, T59, T60, T61, T62, T63, T64, T65, T66, T67, T68, T69, T70, T71, T72, T73, T74, T75, T76, T77, T78, T79, T80, T81, T82, T83, T84, T85, T86, T87, T88, T89, T90, T91, T92, T93, T94, T95, T96, T97, T98, T99, T100, T101, T102, T103, T104, T105, T106, T107>(
        this BaseEnumExt<TEnum, T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15, T16, T17, T18, T19, T20, T21, T22, T23, T24, T25, T26, T27, T28, T29, T30, T31, T32, T33, T34, T35, T36, T37, T38, T39, T40, T41, T42, T43, T44, T45, T46, T47, T48, T49, T50, T51, T52, T53, T54, T55, T56, T57, T58, T59, T60, T61, T62, T63, T64, T65, T66, T67, T68, T69, T70, T71, T72, T73, T74, T75, T76, T77, T78, T79, T80, T81, T82, T83, T84, T85, T86, T87, T88, T89, T90, T91, T92, T93, T94, T95, T96, T97, T98, T99, T100, T101, T102, T103, T104, T105, T106, T107> baseEnumExt)
        where TEnum : Enum where T0 : IType, new() where T1 : IType, new() where T2 : IType, new() where T3 : IType, new() where T4 : IType, new() where T5 : IType, new() where T6 : IType, new() where T7 : IType, new() where T8 : IType, new() where T9 : IType, new() where T10 : IType, new() where T11 : IType, new() where T12 : IType, new() where T13 : IType, new() where T14 : IType, new() where T15 : IType, new() where T16 : IType, new() where T17 : IType, new() where T18 : IType, new() where T19 : IType, new() where T20 : IType, new() where T21 : IType, new() where T22 : IType, new() where T23 : IType, new() where T24 : IType, new() where T25 : IType, new() where T26 : IType, new() where T27 : IType, new() where T28 : IType, new() where T29 : IType, new() where T30 : IType, new() where T31 : IType, new() where T32 : IType, new() where T33 : IType, new() where T34 : IType, new() where T35 : IType, new() where T36 : IType, new() where T37 : IType, new() where T38 : IType, new() where T39 : IType, new() where T40 : IType, new() where T41 : IType, new() where T42 : IType, new() where T43 : IType, new() where T44 : IType, new() where T45 : IType, new() where T46 : IType, new() where T47 : IType, new() where T48 : IType, new() where T49 : IType, new() where T50 : IType, new() where T51 : IType, new() where T52 : IType, new() where T53 : IType, new() where T54 : IType, new() where T55 : IType, new() where T56 : IType, new() where T57 : IType, new() where T58 : IType, new() where T59 : IType, new() where T60 : IType, new() where T61 : IType, new() where T62 : IType, new() where T63 : IType, new() where T64 : IType, new() where T65 : IType, new() where T66 : IType, new() where T67 : IType, new() where T68 : IType, new() where T69 : IType, new() where T70 : IType, new() where T71 : IType, new() where T72 : IType, new() where T73 : IType, new() where T74 : IType, new() where T75 : IType, new() where T76 : IType, new() where T77 : IType, new() where T78 : IType, new() where T79 : IType, new() where T80 : IType, new() where T81 : IType, new() where T82 : IType, new() where T83 : IType, new() where T84 : IType, new() where T85 : IType, new() where T86 : IType, new() where T87 : IType, new() where T88 : IType, new() where T89 : IType, new() where T90 : IType, new() where T91 : IType, new() where T92 : IType, new() where T93 : IType, new() where T94 : IType, new() where T95 : IType, new() where T96 : IType, new() where T97 : IType, new() where T98 : IType, new() where T99 : IType, new() where T100 : IType, new() where T101 : IType, new() where T102 : IType, new() where T103 : IType, new() where T104 : IType, new() where T105 : IType, new() where T106 : IType, new() where T107 : IType, new()
    {
        EnsureArg.IsNotNull(baseEnumExt, nameof(baseEnumExt));

        var baseEnumRust = new BaseEnumRust<TEnum>();
        baseEnumRust.AddTypeDecoder<T0>((TEnum)(object)0);
        baseEnumRust.AddTypeDecoder<T1>((TEnum)(object)1);
        baseEnumRust.AddTypeDecoder<T2>((TEnum)(object)2);
        baseEnumRust.AddTypeDecoder<T3>((TEnum)(object)3);
        baseEnumRust.AddTypeDecoder<T4>((TEnum)(object)4);
        baseEnumRust.AddTypeDecoder<T5>((TEnum)(object)5);
        baseEnumRust.AddTypeDecoder<T6>((TEnum)(object)6);
        baseEnumRust.AddTypeDecoder<T7>((TEnum)(object)7);
        baseEnumRust.AddTypeDecoder<T8>((TEnum)(object)8);
        baseEnumRust.AddTypeDecoder<T9>((TEnum)(object)9);
        baseEnumRust.AddTypeDecoder<T10>((TEnum)(object)10);
        baseEnumRust.AddTypeDecoder<T11>((TEnum)(object)11);
        baseEnumRust.AddTypeDecoder<T12>((TEnum)(object)12);
        baseEnumRust.AddTypeDecoder<T13>((TEnum)(object)13);
        baseEnumRust.AddTypeDecoder<T14>((TEnum)(object)14);
        baseEnumRust.AddTypeDecoder<T15>((TEnum)(object)15);
        baseEnumRust.AddTypeDecoder<T16>((TEnum)(object)16);
        baseEnumRust.AddTypeDecoder<T17>((TEnum)(object)17);
        baseEnumRust.AddTypeDecoder<T18>((TEnum)(object)18);
        baseEnumRust.AddTypeDecoder<T19>((TEnum)(object)19);
        baseEnumRust.AddTypeDecoder<T20>((TEnum)(object)20);
        baseEnumRust.AddTypeDecoder<T21>((TEnum)(object)21);
        baseEnumRust.AddTypeDecoder<T22>((TEnum)(object)22);
        baseEnumRust.AddTypeDecoder<T23>((TEnum)(object)23);
        baseEnumRust.AddTypeDecoder<T24>((TEnum)(object)24);
        baseEnumRust.AddTypeDecoder<T25>((TEnum)(object)25);
        baseEnumRust.AddTypeDecoder<T26>((TEnum)(object)26);
        baseEnumRust.AddTypeDecoder<T27>((TEnum)(object)27);
        baseEnumRust.AddTypeDecoder<T28>((TEnum)(object)28);
        baseEnumRust.AddTypeDecoder<T29>((TEnum)(object)29);
        baseEnumRust.AddTypeDecoder<T30>((TEnum)(object)30);
        baseEnumRust.AddTypeDecoder<T31>((TEnum)(object)31);
        baseEnumRust.AddTypeDecoder<T32>((TEnum)(object)32);
        baseEnumRust.AddTypeDecoder<T33>((TEnum)(object)33);
        baseEnumRust.AddTypeDecoder<T34>((TEnum)(object)34);
        baseEnumRust.AddTypeDecoder<T35>((TEnum)(object)35);
        baseEnumRust.AddTypeDecoder<T36>((TEnum)(object)36);
        baseEnumRust.AddTypeDecoder<T37>((TEnum)(object)37);
        baseEnumRust.AddTypeDecoder<T38>((TEnum)(object)38);
        baseEnumRust.AddTypeDecoder<T39>((TEnum)(object)39);
        baseEnumRust.AddTypeDecoder<T40>((TEnum)(object)40);
        baseEnumRust.AddTypeDecoder<T41>((TEnum)(object)41);
        baseEnumRust.AddTypeDecoder<T42>((TEnum)(object)42);
        baseEnumRust.AddTypeDecoder<T43>((TEnum)(object)43);
        baseEnumRust.AddTypeDecoder<T44>((TEnum)(object)44);
        baseEnumRust.AddTypeDecoder<T45>((TEnum)(object)45);
        baseEnumRust.AddTypeDecoder<T46>((TEnum)(object)46);
        baseEnumRust.AddTypeDecoder<T47>((TEnum)(object)47);
        baseEnumRust.AddTypeDecoder<T48>((TEnum)(object)48);
        baseEnumRust.AddTypeDecoder<T49>((TEnum)(object)49);
        baseEnumRust.AddTypeDecoder<T50>((TEnum)(object)50);
        baseEnumRust.AddTypeDecoder<T51>((TEnum)(object)51);
        baseEnumRust.AddTypeDecoder<T52>((TEnum)(object)52);
        baseEnumRust.AddTypeDecoder<T53>((TEnum)(object)53);
        baseEnumRust.AddTypeDecoder<T54>((TEnum)(object)54);
        baseEnumRust.AddTypeDecoder<T55>((TEnum)(object)55);
        baseEnumRust.AddTypeDecoder<T56>((TEnum)(object)56);
        baseEnumRust.AddTypeDecoder<T57>((TEnum)(object)57);
        baseEnumRust.AddTypeDecoder<T58>((TEnum)(object)58);
        baseEnumRust.AddTypeDecoder<T59>((TEnum)(object)59);
        baseEnumRust.AddTypeDecoder<T60>((TEnum)(object)60);
        baseEnumRust.AddTypeDecoder<T61>((TEnum)(object)61);
        baseEnumRust.AddTypeDecoder<T62>((TEnum)(object)62);
        baseEnumRust.AddTypeDecoder<T63>((TEnum)(object)63);
        baseEnumRust.AddTypeDecoder<T64>((TEnum)(object)64);
        baseEnumRust.AddTypeDecoder<T65>((TEnum)(object)65);
        baseEnumRust.AddTypeDecoder<T66>((TEnum)(object)66);
        baseEnumRust.AddTypeDecoder<T67>((TEnum)(object)67);
        baseEnumRust.AddTypeDecoder<T68>((TEnum)(object)68);
        baseEnumRust.AddTypeDecoder<T69>((TEnum)(object)69);
        baseEnumRust.AddTypeDecoder<T70>((TEnum)(object)70);
        baseEnumRust.AddTypeDecoder<T71>((TEnum)(object)71);
        baseEnumRust.AddTypeDecoder<T72>((TEnum)(object)72);
        baseEnumRust.AddTypeDecoder<T73>((TEnum)(object)73);
        baseEnumRust.AddTypeDecoder<T74>((TEnum)(object)74);
        baseEnumRust.AddTypeDecoder<T75>((TEnum)(object)75);
        baseEnumRust.AddTypeDecoder<T76>((TEnum)(object)76);
        baseEnumRust.AddTypeDecoder<T77>((TEnum)(object)77);
        baseEnumRust.AddTypeDecoder<T78>((TEnum)(object)78);
        baseEnumRust.AddTypeDecoder<T79>((TEnum)(object)79);
        baseEnumRust.AddTypeDecoder<T80>((TEnum)(object)80);
        baseEnumRust.AddTypeDecoder<T81>((TEnum)(object)81);
        baseEnumRust.AddTypeDecoder<T82>((TEnum)(object)82);
        baseEnumRust.AddTypeDecoder<T83>((TEnum)(object)83);
        baseEnumRust.AddTypeDecoder<T84>((TEnum)(object)84);
        baseEnumRust.AddTypeDecoder<T85>((TEnum)(object)85);
        baseEnumRust.AddTypeDecoder<T86>((TEnum)(object)86);
        baseEnumRust.AddTypeDecoder<T87>((TEnum)(object)87);
        baseEnumRust.AddTypeDecoder<T88>((TEnum)(object)88);
        baseEnumRust.AddTypeDecoder<T89>((TEnum)(object)89);
        baseEnumRust.AddTypeDecoder<T90>((TEnum)(object)90);
        baseEnumRust.AddTypeDecoder<T91>((TEnum)(object)91);
        baseEnumRust.AddTypeDecoder<T92>((TEnum)(object)92);
        baseEnumRust.AddTypeDecoder<T93>((TEnum)(object)93);
        baseEnumRust.AddTypeDecoder<T94>((TEnum)(object)94);
        baseEnumRust.AddTypeDecoder<T95>((TEnum)(object)95);
        baseEnumRust.AddTypeDecoder<T96>((TEnum)(object)96);
        baseEnumRust.AddTypeDecoder<T97>((TEnum)(object)97);
        baseEnumRust.AddTypeDecoder<T98>((TEnum)(object)98);
        baseEnumRust.AddTypeDecoder<T99>((TEnum)(object)99);
        baseEnumRust.AddTypeDecoder<T100>((TEnum)(object)100);
        baseEnumRust.AddTypeDecoder<T101>((TEnum)(object)101);
        baseEnumRust.AddTypeDecoder<T102>((TEnum)(object)102);
        baseEnumRust.AddTypeDecoder<T103>((TEnum)(object)103);
        baseEnumRust.AddTypeDecoder<T104>((TEnum)(object)104);
        baseEnumRust.AddTypeDecoder<T105>((TEnum)(object)105);
        baseEnumRust.AddTypeDecoder<T106>((TEnum)(object)106);
        baseEnumRust.AddTypeDecoder<T107>((TEnum)(object)107);

        var idx = 0;
        baseEnumRust.Decode(baseEnumExt.Bytes, ref idx);

        return baseEnumRust;
    }
}
