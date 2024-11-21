﻿#pragma warning disable RS1035 // Do not use APIs banned for analyzers

namespace Sails.ClientGenerator.Loader;

/// <summary>
/// Exposes functionality for loading native libraries and function pointers.
/// </summary>
internal abstract class LibraryLoader
{
    /// <summary>
    /// Loads a native library by name and returns an operating system handle to it.
    /// </summary>
    /// <param name="names">An ordered list of names. Each name is tried in turn, until the library is successfully loaded. </param>
    /// <returns>The operating system handle for the shared library.</returns>
    /// <exception cref="FileNotFoundException"></exception>
    /// <exception cref="ArgumentException"></exception>
    internal IntPtr LoadNativeLibraryByPath(params string[] names)
    {
        foreach (var loadTarget in names)
        {
            if (File.Exists(loadTarget))
            {
                var ret = this.CoreLoadNativeLibrary(loadTarget);
                if (ret != IntPtr.Zero)
                {
                    return ret;
                }
            }
        }
        return IntPtr.Zero;
    }

    /// <summary>
    /// Loads a function pointer out of the given library by name.
    /// </summary>
    /// <param name="handle">The operating system handle of the opened shared library.</param>
    /// <param name="functionName">The name of the exported function to load.</param>
    /// <returns>A pointer to the loaded function.</returns>
    internal IntPtr LoadFunctionPointer(IntPtr handle, string functionName)
    {
        if (string.IsNullOrEmpty(functionName))
        {
            throw new ArgumentException("Parameter must not be null or empty.", nameof(functionName));
        }

        return this.CoreLoadFunctionPointer(handle, functionName);
    }

    /// <summary>
    /// Frees the library represented by the given operating system handle.
    /// </summary>
    /// <param name="handle">The handle of the open shared library.</param>
    internal void FreeNativeLibrary(IntPtr handle)
    {
        if (handle == IntPtr.Zero)
        {
            throw new ArgumentException("Parameter must not be zero.", nameof(handle));
        }

        this.CoreFreeNativeLibrary(handle);
    }

    /// <summary>
    /// Loads a native library by name and returns an operating system handle to it.
    /// </summary>
    /// <param name="name">The name of the library to open. This parameter must not be null or empty.</param>
    /// <returns>The operating system handle for the shared library.
    /// If the library cannot be loaded, IntPtr.Zero should be returned.</returns>
    protected abstract IntPtr CoreLoadNativeLibrary(string name);

    /// <summary>
    /// Frees the library represented by the given operating system handle.
    /// </summary>
    /// <param name="handle">The handle of the open shared library. This must not be zero.</param>
    protected abstract void CoreFreeNativeLibrary(IntPtr handle);

    /// <summary>
    /// Loads a function pointer out of the given library by name.
    /// </summary>
    /// <param name="handle">The operating system handle of the opened shared library. This must not be zero.</param>
    /// <param name="functionName">The name of the exported function to load. This must not be null or empty.</param>
    /// <returns>A pointer to the loaded function.</returns>
    protected abstract IntPtr CoreLoadFunctionPointer(IntPtr handle, string functionName);

    /// <summary>
    /// Returns a default library loader for the running operating system.
    /// </summary>
    /// <returns>A LibraryLoader suitable for loading libraries.</returns>
    public static LibraryLoader GetPlatformDefaultLoader()
    {
        if (RuntimeInformation.IsOSPlatform(OSPlatform.Windows))
        {
            return new Win32LibraryLoader();
        }
        else if (RuntimeInformation.IsOSPlatform(OSPlatform.Linux) || RuntimeInformation.IsOSPlatform(OSPlatform.OSX))
        {
            return new UnixLibraryLoader();
        }

        throw new PlatformNotSupportedException("This platform cannot load native libraries.");
    }

    private class Win32LibraryLoader : LibraryLoader
    {
        protected override void CoreFreeNativeLibrary(IntPtr handle)
        {
            Kernel32.FreeLibrary(handle);
        }

        protected override IntPtr CoreLoadFunctionPointer(IntPtr handle, string functionName)
        {
            return Kernel32.GetProcAddress(handle, functionName);
        }

        protected override IntPtr CoreLoadNativeLibrary(string name)
        {
            return Kernel32.LoadLibrary(name);
        }
    }

    private class UnixLibraryLoader : LibraryLoader
    {
        protected override void CoreFreeNativeLibrary(IntPtr handle)
        {
            Libdl.dlclose(handle);
        }

        protected override IntPtr CoreLoadFunctionPointer(IntPtr handle, string functionName)
        {
            return Libdl.dlsym(handle, functionName);
        }

        protected override IntPtr CoreLoadNativeLibrary(string name)
        {
            return Libdl.dlopen(name, Libdl.RTLD_NOW);
        }
    }
}
