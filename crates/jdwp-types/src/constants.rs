//! constants

use crate::macros::tagged_type;
use bitfield::bitfield;

tagged_type! {
    repr: u16;
    /// An error constant
    #[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
    pub enum ErrorConstant {
        /// No error has occurred.
        None = 0,
        /// Passed thread is null, is not a valid thread or has exited.
        InvalidThread = 10,
        /// Thread group invalid.
        InvalidThreadGroup = 11,
        /// Invalid priority.
        InvalidPriority = 12,
        /// If the specified thread has not been suspended by an event.
        ThreadNotSuspended = 13,
        /// Thread already suspended.
        ThreadSuspended = 14,
        /// Thread has not been started or is now dead.
        ThreadNotAlive = 15,
        /// If this reference type has been unloaded and garbage collected.
        InvalidObject = 20,
        /// Invalid class.
        InvalidClass = 21,
        /// Class has been loaded but not yet prepared.
        ClassNotPrepared = 22,
        /// Invalid method.
        InvalidMethodId = 23,
        /// Invalid location.
        InvalidLocation = 24,
        /// Invalid field.
        InvalidFieldId = 25,
        /// Invalid jframeID.
        InvalidFrameId = 30,
        /// There are no more Java or JNI frames on the call stack.
        NoMoreFrames = 31,
        /// Information about the frame is not available.
        OpaqueFrame = 32,
        /// Operation can only be performed on current frame.
        NotCurrentFrame = 33,
        /// The variable is not an appropriate type for the function used.
        TypeMismatch = 34,
        /// Invalid slot.
        InvalidSlot = 35,
        /// Item already set.
        Duplicate = 40,
        /// Desired element not found.
        NotFound = 41,
        /// Invalid monitor.
        InvalidMonitor = 50,
        /// This thread doesn't own the monitor.
        NotMonitorOwner = 51,
        /// The call has been interrupted before completion.
        Interrupt = 52,
        /// The virtual machine attempted to read a class file and determined that the file is malformed or otherwise cannot be interpreted as a class file.
        InvalidClassFormat = 60,
        /// A circularity has been detected while initializing a class.
        CircularClassDefinition = 61,
        /// The verifier detected that a class file, though well formed, contained some sort of internal inconsistency or security problem.
        FailsVerification = 62,
        /// Adding methods has not been implemented.
        AddMethodNotImplemented = 63,
        /// Schema change has not been implemented.
        SchemaChangeNotImplemented = 64,
        /// The state of the thread has been modified, and is now inconsistent.
        InvalidTypestate = 65,
        /// A direct superclass is different for the new class version, or the set of directly implemented interfaces is different and canUnrestrictedlyRedefineClasses is false.
        HierarchyChangeNotImplemented = 66,
        /// The new class version does not declare a method declared in the old class version and canUnrestrictedlyRedefineClasses is false.
        DeleteMethodNotImplemented = 67,
        /// A class file has a version number not supported by this VM.
        UnsupportedVersion = 68,
        /// The class name defined in the new class file is different from the name in the old class object.
        NamesDontMatch = 69,
        /// The new class version has different modifiers and and canUnrestrictedlyRedefineClasses is false.
        ClassModifiersChangeNotImplemented = 70,
        /// A method in the new class version has different modifiers than its counterpart in the old class version and and canUnrestrictedlyRedefineClasses is false.
        MethodModifiersChangeNotImplemented = 71,
        /// The functionality is not implemented in this virtual machine.
        NotImplemented = 99,
        /// Invalid pointer.
        NullPointer = 100,
        /// Desired information is not available.
        AbsentInformation = 101,
        /// The specified event type id is not recognized.
        InvalidEventType = 102,
        /// Illegal argument.
        IllegalArgument = 103,
        /// The function needed to allocate memory and no more memory was available for allocation.
        OutOfMemory = 110,
        /// Debugging has not been enabled in this virtual machine. JVMTI cannot be used.
        AccessDenied = 111,
        /// The virtual machine is not running.
        VmDead = 112,
        /// An unexpected internal error has occurred.
        Internal = 113,
        /// The thread being used to call this function is not attached to the virtual machine. Calls must be made from attached threads.
        UnattachedThread = 115,
        /// object type id or class tag.
        InvalidTag = 500,
        /// Previous invoke not complete.
        AlreadyInvoking = 502,
        /// Index is invalid.
        InvalidIndex = 503,
        /// The length is invalid.
        InvalidLength = 504,
        /// The string is invalid.
        InvalidString = 506,
        /// The class loader is invalid.
        InvalidClassLoader = 507,
        /// The array is invalid.
        InvalidArray = 508,
        /// Unable to load the transport.
        TransportLoad = 509,
        /// Unable to initialize the transport.
        TransportInit = 510,
        /// INVALID_COUNT	512	The count is invalid.
        NativeMethod = 511,
    }
}

tagged_type! {
    /// A tag for a certain reference type
    #[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
    pub enum TypeTag {
        /// ReferenceType is a class.
        Class = 1,
        /// ReferenceType is an interface.
        Interface = 2,
        /// Reference type is an array.
        Array = 3
    }
}

tagged_type! {
    /// A tag for a certain type
    #[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
    pub enum Tag {
        /// '[' - an array object (objectID size).
        Array = 91,
        /// 'B' - a byte value (1 byte).
        Byte = 66,
        /// 'C' - a character value (2 bytes).
        Char = 67,
        /// 'L' - an object (objectID size).
        Object = 76,
        /// 'F' - a float value (4 bytes).
        Float = 70,
        /// 'D' - a double value (8 bytes).
        Double = 68,
        /// 'I' - an int value (4 bytes).
        Int = 73,
        /// 'J' - a long value (8 bytes).
        Long = 74,
        /// 'S' - a short value (2 bytes).
        Short = 83,
        /// 'V' - a void value (no bytes).
        Void = 86,
        /// 'Z' - a boolean value (1 byte).
        Boolean = 90,
        /// 's' - a String object (objectID size).
        String = 115,
        /// 't' - a Thread object (objectID size).
        Thread = 116,
        /// 'g' - a ThreadGroup object (objectID size).
        ThreadGroup = 103,
        /// 'l' - a ClassLoader object (objectID size).
        ClassLoader = 108,
        /// 'c' - a class object object (objectID size).
        ClassObject = 99,
    }
}

tagged_type! {
    /// Suspension policy for the event
    #[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
    pub enum SuspendPolicy {
        /// Nothing in the VM was suspended
        None = 0,
        /// Only the thread where the event started is suspended
        EventThread = 1,
        /// All threads are suspended
        All = 2
    }
}

bitfield! {
    #[derive(Clone, Copy)]
    pub struct ClassStatus(u32);
    impl Debug;

    pub verified, _: 0;
    pub prepared, _: 1;
    pub initialized, _: 2;
    pub error, _: 3;
}

tagged_type! {
    /// Suspension policy for the event
    #[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
    pub enum EventKind {
        SingleStep = 1,
        Breakpoint = 2,
        FramePop = 3,
        Exception = 4,
        UserDefined = 5,
        ThreadStart = 6,
        ThreadDeath = 7,
        ClassPrepare = 8,
        ClassUnload = 9,
        ClassLoad = 10,
        FieldAccess = 20,
        FieldModification = 21,
        ExceptionCatch = 30,
        MethodEntry = 40,
        MethodExit = 41,
        MethodExitWithReturnValue = 42,
        MonitorContendedEnter = 43,
        MonitorContendedEntered = 44,
        MonitorWait = 45,
        MonitorWaited = 46,
        VmStart = 90,
        VmDeath = 99,
        /// Never sent across JDWP
        VmDisconnected = 100,
    }
}
