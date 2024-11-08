#pragma warning disable RS1035
using System.Reflection;

namespace Sails.ClientGenerator;

internal static unsafe partial class NativeMethods
{
    internal static IntPtr LoadNativeLibrary()
    {
        // Determine where to extract the DLL
        var tempDirectory = Path.Combine(Path.GetTempPath(), __DllName);
        Directory.CreateDirectory(tempDirectory);

        var nativeLibraryPath = Path.Combine(tempDirectory, __DllName + ".dll");
        // Extract the DLL only if it doesn't already exist
        if (!File.Exists(nativeLibraryPath))
        {
            ExtractResourceToFile(__DllName + ".dll", nativeLibraryPath);
        }

        //#if DEBUG
        //            var combinedPath = Path.Combine(AppContext.BaseDirectory, __DllName);
        //            if (File.Exists(combinedPath) || File.Exists(combinedPath + ".dll"))
        //            {
        //                return LibraryLoader.GetPlatformDefaultLoader().LoadNativeLibrary(__DllName);
        //            }
        //#endif

        //    var path = "runtimes/";
        //    var extension = "";

        //    if (RuntimeInformation.IsOSPlatform(OSPlatform.Windows))
        //    {
        //        path += "win-";
        //        extension = ".dll";
        //    }
        //    else if (RuntimeInformation.IsOSPlatform(OSPlatform.OSX))
        //    {
        //        path += "osx-";
        //        extension = ".dylib";
        //    }
        //    else
        //    {
        //        path += "linux-";
        //        extension = ".so";
        //    }

        //    if (RuntimeInformation.OSArchitecture == Architecture.X86)
        //    {
        //        path += "x86";
        //    }
        //    else if (RuntimeInformation.OSArchitecture == Architecture.X64)
        //    {
        //        path += "x64";
        //    }
        //    else if (RuntimeInformation.OSArchitecture == Architecture.Arm64)
        //    {
        //        path += "arm64";
        //    }

        //    path += "/native/" + __DllName + extension;

        //    return LibraryLoader.GetPlatformDefaultLoader().LoadNativeLibrary(__DllName);
        //}
        return LibraryLoader.GetPlatformDefaultLoader().LoadNativeLibrary(__DllName, new TempPathResolver(nativeLibraryPath));
        //return IntPtr.Zero;
    }

    private static void ExtractResourceToFile(string resourceName, string filePath)
    {
        var assembly = Assembly.GetExecutingAssembly();
        var resourceFullName = $"{assembly.GetName().Name}.{resourceName}";

        using (var resourceStream = assembly.GetManifestResourceStream(resourceFullName))
        {
            if (resourceStream == null)
            {
                throw new Exception($"Resource '{resourceFullName}' not found in assembly.");
            }
            using (var fileStream = new FileStream(filePath, FileMode.Create, FileAccess.Write))
            {
                resourceStream.CopyTo(fileStream);
            }
        }
    }
}
