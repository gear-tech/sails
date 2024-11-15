namespace Sails.ClientGenerator.Loader;
internal class NativeLibrary : IDisposable
{
    private static readonly LibraryLoader Loader = LibraryLoader.GetPlatformDefaultLoader();

    /// <summary>
    /// The operating system handle of the loaded library.
    /// </summary>
    public IntPtr Handle { get; }

    /// <summary>
    /// Constructs a new NativeLibrary using the platform's default library loader.
    /// </summary>
    /// <param name="path">The path to the library to load.</param>
    public NativeLibrary(string path)
    {
        var libPtr = Loader.LoadNativeLibraryByPath(path);
        if (libPtr == IntPtr.Zero)
        {
            throw new FileNotFoundException($"Could not find or load the native library: {path}");
        }
        this.Handle = libPtr;
    }

    /// <summary>
    /// Loads a function whose signature matches the given delegate type's signature.
    /// </summary>
    /// <typeparam name="T">The type of delegate to return.</typeparam>
    /// <param name="name">The name of the native export.</param>
    /// <returns>A delegate wrapping the native function.</returns>
    /// <exception cref="InvalidOperationException">Thrown when no function with the given name
    /// is exported from the native library.</exception>
    public T LoadFunction<T>(string name)
    {
        var functionPtr = Loader.LoadFunctionPointer(this.Handle, name);
        if (functionPtr == IntPtr.Zero)
        {
            throw new InvalidOperationException($"No function was found with the name {name}.");
        }
        return Marshal.GetDelegateForFunctionPointer<T>(functionPtr);
    }

    /// <summary>
    /// Loads a function pointer with the given name.
    /// </summary>
    /// <param name="name">The name of the native export.</param>
    /// <returns>A function pointer for the given name, or 0 if no function with that name exists.</returns>
    public IntPtr LoadFunction(string name) => Loader.LoadFunctionPointer(this.Handle, name);

    /// <summary>
    /// Frees the native library. Function pointers retrieved from this library will be void.
    /// </summary>
    public void Dispose() => Loader.FreeNativeLibrary(this.Handle);
}
