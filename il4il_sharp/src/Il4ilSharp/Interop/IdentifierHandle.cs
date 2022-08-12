namespace Il4ilSharp.Interop;

using System;
using System.Text;
using System.Threading;
using Il4ilSharp.Interop.Native;

/// <summary>
/// Provides a thread-safe wrapper for an IL4IL identifier string.
/// </summary>
/// <seealso cref="IdentifierString"/>
public unsafe sealed class IdentifierHandle : IDisposable {
    private readonly object sync = new();

    private Identifier.Opaque* identifier;

    public bool IsDisposed => identifier == null;

    private void EnterLock() => Monitor.Enter(sync);

    private void ExitLock() => Monitor.Exit(sync);

    private ReadOnlySpan<byte> AsReadOnlySpan() {
        if (IsDisposed) {
            return default;
        }

        nuint length;
        Error.Opaque* error;
        byte* contents = Identifier.Contents(identifier, out length, out error);
        // TODO: Throw error
        return new ReadOnlySpan<byte>(contents, (int)length);
    }

    /// <summary>Attempts to convert the identifier string to a .NET <see cref="String"/>.</summary>
    /// <returns>The contents of the identifier string, or <see cref="String.Empty"/> if the identifier was disposed.</returns>
    public override string ToString() {
        if (IsDisposed) {
            return String.Empty;
        }

        try {
            EnterLock();
            return Encoding.UTF8.GetString(AsReadOnlySpan());
        } finally {
            ExitLock();
        }
    }

    /// <summary>Attempts to copy the UTF-8 contents of the identifier string into a newly allocated byte array.</summary>
    /// <returns>An array containing the UTF-8 string, or an empty array if the identifier was disposed.</returns>
    public byte[] ToArray() {
        if (IsDisposed) {
            return Array.Empty<byte>();
        }

        try {
            EnterLock();
            return AsReadOnlySpan().ToArray();
        } finally {
            ExitLock();
        }
    }

    private void Dispose(bool disposing) {
        try {
            if (disposing) {
                EnterLock();
            }

            if (!IsDisposed) {
                Error.Opaque* error;
                Identifier.Dispose(identifier, out error);
                identifier = null;
                // TODO: Throw error
            }
        } finally {
            if (disposing) {
                ExitLock();
            }
        }
    }

    /// <inheritdoc/>
    public void Dispose() {
        Dispose(true);
        GC.SuppressFinalize(this);
    }

    /// <summary>Calls native code to disposing the underlying identifier string.</summary>
    ~IdentifierHandle() => Dispose(false);
}
