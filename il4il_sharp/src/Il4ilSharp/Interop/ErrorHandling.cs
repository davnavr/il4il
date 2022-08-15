namespace Il4ilSharp.Interop;

using System;
using System.Buffers;
using System.Text;
using Il4ilSharp.Interop.Native;

internal unsafe static class ErrorHandling {
    internal static void Throw(Error.Opaque* error) {
        if (error != null) {
            byte[]? rented = null;

            try {
                int length = (int)Error.MessageLength(error);
                Span<byte> buffer = length <= 512 ? stackalloc byte[length] : new Span<byte>(rented = ArrayPool<byte>.Shared.Rent(length), 0, length);
                fixed (byte* bytes = buffer) {
                    Error.MessageCopyTo(error, bytes);
                }

                throw new Il4ilSharp.ErrorHandlingException(Encoding.UTF8.GetString(buffer));
            } finally {
                if (rented != null) {
                    ArrayPool<byte>.Shared.Return(rented);
                }

                Error.Dispose(error);
            }
        }
    }
}
