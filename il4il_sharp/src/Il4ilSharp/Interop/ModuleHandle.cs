namespace Il4ilSharp.Interop;

using System;
using Il4ilSharp.Interop.Native;

/// <summary>
/// Provides a thread-safe wrapper for an in-memory representation of an IL4IL module.
/// </summary>
public unsafe sealed class ModuleHandle : SynchronizedHandle<Module.Opaque> {
    /// <summary>Gets a value indicating whether the module was already disposed.</summary>
    public new bool IsDisposed => base.IsDisposed;

    internal ModuleHandle(Module.Opaque* module) : base(module) { }

    /// <summary>
    /// Initializes a new <see cref="ModuleHandle"/>, allocating a new module.
    /// </summary>
    public ModuleHandle() : this(Module.Create()) { }

    /// <summary>Adds a module name to a metadata section within the module.</summary>
    /// <exception cref="ArgumentNullException">Thrown if the <paramref name="name"/> is <see langword="null"/>.</exception>
    /// <exception cref="ObjectDisposedException">Thrown if the <paramref name="name"/> or module was already disposed.</exception>
    public void AddMetadataName(IdentifierHandle name) {
        ArgumentNullException.ThrowIfNull(name);

        try {
            var module = Lock();
            try {
                var moduleNamePointer = name.Lock();
                Module.AddMetadataName(module, moduleNamePointer);
            } finally {
                name.Unlock();
            }
        } finally {
            Unlock();
        }
    }

    /// <summary>Performs validation on the module, and disposes this handle.</summary>
    /// <exception cref="ObjectDisposedException">Thrown if the module was already disposed.</exception>
    public BrowserHandle ValidateAndDispose() {
        Browser.Opaque* browser;
        Module.ValidateAndDispose(Take(), out browser);
        return new BrowserHandle(browser);
    }

    /// <summary>Writes the binary contents of the module to the file at the specified <paramref name="path"/>.</summary>
    /// <exception cref="ArgumentNullException">Thrown if the <paramref name="path"/> was <see langword="null"/>.</exception>
    /// <exception cref="ObjectDisposedException">Thrown if the module or <paramref name="path"/> was already disposed.</exception>
    public void WriteBinaryToPath(IdentifierHandle path) {
        ArgumentNullException.ThrowIfNull(path);

        try {
            var module = Lock();
            try {
                Identifier.Opaque* destination = path.Lock();
                ErrorHandling.Throw(Module.WriteBinaryToPath(module, destination));
            } finally {
                path.Unlock();
            }
        } finally {
            Unlock();
        }
    }

    /// <summary>Writes the binary contents of the module to the file using the specified <paramref name="writer"/>.</summary>
    /// <remarks>
    /// As any exceptions thrown by the <paramref name="writer"/> might result in undefined behavior, it is recommended to use
    /// <see cref="IByteWriter"/> instead.
    /// </remarks>
    /// <exception cref="ArgumentNullException">Thrown if the <paramref name="writer"/> was <see langword="null"/>.</exception>
    /// <exception cref="ObjectDisposedException">Thrown if the module was already disposed.</exception>
    public void WriteBinary(Native.ByteWriter writer) {
        ArgumentNullException.ThrowIfNull(writer);
        try {
            Module.Opaque* module = Lock();
            Module.WriteBinary(module, writer);
        } finally {
            Unlock();
        }
    }

    /// <summary>Writes the binary contents of the module to the file into the <paramref name="destination"/>.</summary>
    /// <exception cref="ArgumentNullException">Thrown if the <paramref name="destination"/> was <see langword="null"/>.</exception>
    public void WriteBinary<W>(W destination) where W : IByteWriter {
        ArgumentNullException.ThrowIfNull(destination);
        CaughtException catcher;
        Native.ByteWriter writer = destination.CreateNativeWriter(out catcher);

        try {
            WriteBinary(writer);
        } catch (Exception e) {
            if (catcher.Thrown != null) {
                catcher.Catch(e);
                catcher.ThrowAny();
            } else {
                // This is probably unreachable, but this rethrows the exception if one was somehow not caught by the catcher 
                throw;
            }
        }

        // Might not be necessary, but checks if (somehow) an exception was missed and we need to throw it
        catcher.ThrowAny();
    }

    private protected override unsafe void Cleanup(Module.Opaque* pointer) {
        Error.Opaque* error;
        Module.Dispose(pointer, out error);
        ErrorHandling.Throw(error);
    }
}
