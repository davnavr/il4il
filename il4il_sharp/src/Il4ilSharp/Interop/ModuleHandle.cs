namespace Il4ilSharp.Interop;

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

    private protected override unsafe void Cleanup(Module.Opaque* pointer) {
        Error.Opaque* error;
        Module.Dispose(pointer, out error);
        ErrorHandling.Throw(error);
    }
}
