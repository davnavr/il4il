namespace Il4ilSharp.Interop;

using System;
using System.Buffers;
using System.Text;
using Il4ilSharp.Interop.Native;

/// <summary>
/// Provides a thread-safe wrapper for an IL4IL error message.
/// </summary>
public unsafe sealed class ErrorHandle : SynchronizedHandle<Error.Opaque> {
    internal ErrorHandle(Error.Opaque* message) : base(message) { }

    private static Error.Opaque* Allocate(string message) {
        fixed(char* contents = message) {
            return Error.MessageFromUtf16(contents, (nuint)message.Length);
        }
    }

    /// <summary>Initializes an <see cref="ErrorHandle"/> from a UTF-16 message.</summary>
    public ErrorHandle(string message) : this(Allocate(message ?? String.Empty)) { }

    /// <summary>Gets the contents of the error message as a UTF-16 <see langword="String"/>.</summary>
    /// <returns>A <se cref="String"/> containing the error message, or <see cref="String.Empty"/> if the message was disposed.</returns>
    public override string ToString() {
        if (IsDisposed) {
            return String.Empty;
        }

        byte[]? rented = null;

        try {
            Error.Opaque* message = LockOrNullIfDisposed();
            if (message == null) {
                return String.Empty;
            }

            int length = (int)Error.MessageLength(message);
            Span<byte> buffer = length <= 512 ? stackalloc byte[length] : new Span<byte>(rented = ArrayPool<byte>.Shared.Rent(length), 0, length);
            fixed (byte* bytes = buffer) {
                Error.MessageCopyTo(message, bytes);
            }

            return Encoding.UTF8.GetString(buffer);
        } finally {
            Unlock();
        }
    }

    private protected override unsafe void Cleanup(Error.Opaque* pointer) => Error.Dispose(pointer);
}
