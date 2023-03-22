
#![allow(dead_code, non_snake_case, non_camel_case_types)]
use std::ffi::{CStr, c_char, c_int, c_uint, c_void};

fn main () {
}


/*#[derive(Debug, Copy, Clone)]*/
#[repr(C)]
struct SparseExtentHeader {
  magicNumber: c_int,
  version: c_int,
  flags: c_int,
  capacity: c_int,
  grainSize: c_int,
  descriptorOffset: c_int,
  descriptorSize: c_int,
  numGTEsPerGT: c_int,
  rgdOffset: c_int,
  gdOffset: c_int,
  overHead: c_int,
  uncleanShutdown: bool,
  singleEndLineChar: c_char,
  nonEndLineChar: c_char,
  doubleEndLineChar1: c_char,
  doubleEndLineChar2: c_char,
  compressAlgorithm: c_int,
  pad: [c_int; 433],
}

/*#[derive(Debug, Copy, Clone)]*/
#[repr(C)]
struct VMDKMARKER {
  uSector: c_int,
  cbSize: c_int,
  uType: c_int,
}
type PVMDKMARKER = *const VMDKMARKER ;
enum VMDKETYPE {
    VMDKETYPE_HOSTED_SPARSE,
    VMDKETYPE_FLAT,
    VMDKETYPE_ZERO,
    VMDKETYPE_VMFS
}enum VMDKACCESS {
    VMDKACCESS_NOACCESS,
    VMDKACCESS_READONLY,
    VMDKACCESS_READWRITE
}type PVMDKIMAGE = *const VMDKIMAGE ;

/*#[derive(Debug, Copy, Clone)]*/
#[repr(C)]
struct VMDKFILE {
  pszFilename: Box<CStr>,
  pszBasename: Box<CStr>,
  fOpen: c_uint,
  pStorage: c_int,
  uReferences: c_uint,
  fDelete: bool,
  pImage: PVMDKIMAGE,
  pNext: *const VMDKFILE,
  pPrev: *const VMDKFILE,
}
type PVMDKFILE = *const VMDKFILE ;

/*#[derive(Debug, Copy, Clone)]*/
#[repr(C)]
struct VMDKEXTENT {
  pFile: PVMDKFILE,
  pszBasename: Box<CStr>,
  pszFullname: Box<CStr>,
  cSectors: c_int,
  cSectorsPerGrain: c_int,
  uDescriptorSector: c_int,
  cDescriptorSectors: c_int,
  uSectorGD: c_int,
  uSectorRGD: c_int,
  cOverheadSectors: c_int,
  cNominalSectors: c_int,
  uSectorOffset: c_int,
  cGTEntries: c_int,
  cSectorsPerGDE: c_int,
  cGDEntries: c_int,
  uFreeSector: c_int,
  uExtent: c_int,
  pDescData: *const c_char,
  pGD: *const c_int,
  pRGD: *const c_int,
  uVersion: c_int,
  enmType: VMDKETYPE,
  enmAccess: VMDKACCESS,
  fUncleanShutdown: bool,
  fMetaDirty: bool,
  fFooter: bool,
  uCompression: c_int,
  uAppendPosition: c_int,
  uLastGrainAccess: c_int,
  uGrainSectorAbs: c_int,
  uGrain: c_int,
  cbGrainStreamRead: c_int,
  cbCompGrain: usize,
  pvCompGrain: c_void,
  pvGrain: c_void,
  pImage: *const VMDKIMAGE,
}
type PVMDKEXTENT = *const VMDKEXTENT ;

/*#[derive(Debug, Copy, Clone)]*/
#[repr(C)]
struct VMDKDESCRIPTOR {
  uFirstDesc: c_uint,
  uFirstExtent: c_uint,
  uFirstDDB: c_uint,
  cLines: c_uint,
  cbDescAlloc: usize,
  fDirty: bool,
  aLines: [*const c_char; 1100],
  aNextLines: [c_uint; 1100],
}
type PVMDKDESCRIPTOR = *const VMDKDESCRIPTOR ;

/*#[derive(Debug, Copy, Clone)]*/
#[repr(C)]
struct VMDKGTCACHEENTRY {
  uExtent: c_int,
  uGTBlock: c_int,
  aGTData: [c_int; 128],
}
type PVMDKGTCACHEENTRY = *const VMDKGTCACHEENTRY ;

/*#[derive(Debug, Copy, Clone)]*/
#[repr(C)]
struct VMDKGTCACHE {
  aGTCache: [VMDKGTCACHEENTRY; 256],
  cEntries: c_uint,
}
type PVMDKGTCACHE = *const VMDKGTCACHE ;

/*#[derive(Debug, Copy, Clone)]*/
#[repr(C)]
struct VMDKIMAGE {
  pszFilename: Box<CStr>,
  pFile: PVMDKFILE,
  pVDIfsDisk: c_int,
  pVDIfsImage: c_int,
  pIfError: c_int,
  pIfIo: c_int,
  pExtents: PVMDKEXTENT,
  cExtents: c_uint,
  pFiles: PVMDKFILE,
  paSegments: c_int,
  cSegments: c_uint,
  uOpenFlags: c_uint,
  uImageFlags: c_uint,
  cbSize: c_int,
  PCHSGeometry: c_int,
  LCHSGeometry: c_int,
  ImageUuid: c_int,
  ModificationUuid: c_int,
  ParentUuid: c_int,
  ParentModificationUuid: c_int,
  pGTCache: PVMDKGTCACHE,
  pDescData: *const c_char,
  cbDescAlloc: usize,
  Descriptor: VMDKDESCRIPTOR,
  RegionList: c_int,
}

/*#[derive(Debug, Copy, Clone)]*/
#[repr(C)]
struct VMDKCOMPRESSIO {
  pImage: PVMDKIMAGE,
  iOffset: c_int,
  cbCompGrain: usize,
  pvCompGrain: c_void,
}

/*#[derive(Debug, Copy, Clone)]*/
#[repr(C)]
struct VMDKGRAINALLOCASYNC {
  fIoErr: bool,
  cIoXfersPending: c_uint,
  uSector: c_int,
  fGTUpdateNeeded: bool,
  pExtent: PVMDKEXTENT,
  uGrainOffset: c_int,
  uGTSector: c_int,
  uRGTSector: c_int,
}
type PVMDKGRAINALLOCASYNC = *const VMDKGRAINALLOCASYNC ;

/*#[derive(Debug, Copy, Clone)]*/
#[repr(C)]
struct VMDKRENAMESTATE {
  apszOldName: *const *const c_char,
  apszNewName: *const *const c_char,
  apszNewLines: *const *const c_char,
  pszOldDescName: *const c_char,
  fImageFreed: bool,
  fEmbeddedDesc: bool,
  cExtents: c_uint,
  pszNewBaseName: *const c_char,
  pszOldBaseName: *const c_char,
  pszNewFullName: *const c_char,
  pszOldFullName: *const c_char,
  pszOldImageName: Box<CStr>,
  DescriptorCopy: VMDKDESCRIPTOR,
  ExtentCopy: VMDKEXTENT,
}
type PVMDKRENAMESTATE = *const VMDKRENAMESTATE ;
