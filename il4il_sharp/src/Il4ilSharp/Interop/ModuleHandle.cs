namespace Il4ilSharp.Interop;

using System;
using Il4ilSharp.Interop.Native;

/// <summary>
/// Provides a thread-safe wrapper for an in-memory representation of an IL4IL module.
/// </summary>
public unsafe sealed class ModuleHandle : SyncHandle<Module.Opaque> {
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
            var module = Enter();
            try {
                var moduleNamePointer = name.Enter();
                Module.AddMetadataName(module, moduleNamePointer);
            } finally {
                name.Exit();
            }
        } finally {
            Exit();
        }
    }

    /// <summary>Performs validation on the module, and disposes this handle.</summary>
    /// <exception cref="ObjectDisposedException">Thrown if the module was already disposed.</exception>
    public BrowserHandle ValidateAndDispose() {
        Browser.Opaque* browser;
        Module.ValidateAndDispose(Take(), out browser);
        return new BrowserHandle(browser);
    }

    private protected override unsafe void Cleanup(Module.Opaque* pointer) {
        Error.Opaque* error;
        Module.Dispose(pointer, out error);
        ErrorHandling.Throw(error);
    }
}
