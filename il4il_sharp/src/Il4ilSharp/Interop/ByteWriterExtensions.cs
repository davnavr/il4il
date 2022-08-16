namespace Il4ilSharp.Interop;

using System;

/// <summary>Provides extension methods for <see cref="IByteWriter"/>.</summary>
public static class ByteWriterExtensions {
    /// <summary>
    /// Creates a <see langword="delegate"/> that can be used by the IL4IL C API to write bytes using the specified <paramref name="writer"/>.
    /// </summary>
    /// <remarks>
    /// Any exceptions thrown by the <paramref name="writer"/> will be stored in the <paramref name="catcher"/>. Callers should rethrow
    /// those caught exceptions by calling <see cref="CaughtException.ThrowAny"/>.
    /// </remarks>
    public static Native.ByteWriter CreateNativeWriter<W>(this W writer, out CaughtException catcher) where W : IByteWriter {
        var store = new CaughtException();
        catcher = store;
        unsafe {
            return (byte* buffer, nuint length) => {
                try {
                    writer.Write(new ReadOnlySpan<byte>((void*)buffer, (int)length));
                    return null;
                } catch (Exception e) {
                    store.Catch(e);
                    return new ErrorHandle(e.Message).Take();
                }
            };
        }
    }
}
