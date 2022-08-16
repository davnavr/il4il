namespace Il4ilSharp.Interop.Native;

using System.Runtime.InteropServices;

/// <summary>Methods for manipulating IL4IL error messages.</summary>
public unsafe static class Error {
    public readonly ref struct Opaque { }

    [DllImport(Library.Name, CallingConvention = CallingConvention.Cdecl, EntryPoint = "il4il_error_dispose", ExactSpelling = true)]
    public static extern void Dispose(Opaque* message);

    [DllImport(Library.Name, CallingConvention = CallingConvention.Cdecl, EntryPoint = "il4il_error_message_length", ExactSpelling = true)]
    public static extern nuint MessageLength(Opaque* message);

    [DllImport(Library.Name, CallingConvention = CallingConvention.Cdecl, EntryPoint = "il4il_error_message_copy_to", ExactSpelling = true)]
    public static extern void MessageCopyTo(Opaque* message, byte* buffer);

    [DllImport(Library.Name, CallingConvention = CallingConvention.Cdecl, EntryPoint = "il4il_error_message_from_utf16", ExactSpelling = true)]
    public static extern Opaque* MessageFromUtf16(char* message, nuint length);
}
