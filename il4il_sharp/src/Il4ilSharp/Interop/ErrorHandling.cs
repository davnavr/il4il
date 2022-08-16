namespace Il4ilSharp.Interop;

using System;
using System.Buffers;
using System.Text;
using Il4ilSharp.Interop.Native;

internal unsafe static class ErrorHandling {
    internal static void Throw(Error.Opaque* error) {
        if (error != null) {
            using ErrorHandle handle = new ErrorHandle(error);
            throw new ErrorHandlingException(handle.ToString());
        }
    }
}
