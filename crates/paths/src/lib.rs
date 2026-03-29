//! Thin wrappers around [`camino::Utf8PathBuf`], distinguishing
//! between absolute and relative paths.

#![expect(clippy::disallowed_types, reason = "this crate defines the better type")]

use std::{
    borrow::Borrow,
    ffi::OsStr,
    fmt, ops,
    path::{self, Path, PathBuf},
};

pub use camino::{Utf8Component, Utf8Components, Utf8Path, Utf8PathBuf, Utf8Prefix};

/// A [`Utf8PathBuf`] that is guaranteed to be absolute.
#[derive(Debug, Clone, Ord, PartialOrd, Eq, Hash)]
pub struct AbsPathBuf(Utf8PathBuf);

impl From<AbsPathBuf> for Utf8PathBuf {
    fn from(AbsPathBuf(path_buf): AbsPathBuf) -> Utf8PathBuf {
        path_buf
    }
}

impl From<AbsPathBuf> for PathBuf {
    fn from(AbsPathBuf(path_buf): AbsPathBuf) -> PathBuf {
        path_buf.into()
    }
}

impl ops::Deref for AbsPathBuf {
    type Target = AbsPath;
    fn deref(&self) -> &AbsPath {
        self.as_path()
    }
}

impl AsRef<Utf8Path> for AbsPathBuf {
    fn as_ref(&self) -> &Utf8Path {
        self.0.as_path()
    }
}

impl AsRef<OsStr> for AbsPathBuf {
    fn as_ref(&self) -> &OsStr {
        self.0.as_ref()
    }
}

impl AsRef<Path> for AbsPathBuf {
    fn as_ref(&self) -> &Path {
        self.0.as_ref()
    }
}

impl AsRef<AbsPath> for AbsPathBuf {
    fn as_ref(&self) -> &AbsPath {
        self.as_path()
    }
}

impl Borrow<AbsPath> for AbsPathBuf {
    fn borrow(&self) -> &AbsPath {
        self.as_path()
    }
}

impl TryFrom<Utf8PathBuf> for AbsPathBuf {
    type Error = Utf8PathBuf;
    fn try_from(path_buf: Utf8PathBuf) -> Result<AbsPathBuf, Utf8PathBuf> {
        if !path_buf.is_absolute() {
            return Err(path_buf);
        }
        Ok(AbsPathBuf(path_buf))
    }
}

impl TryFrom<&str> for AbsPathBuf {
    type Error = Utf8PathBuf;
    fn try_from(path: &str) -> Result<AbsPathBuf, Utf8PathBuf> {
        AbsPathBuf::try_from(Utf8PathBuf::from(path))
    }
}

impl TryFrom<String> for AbsPathBuf {
    type Error = Utf8PathBuf;
    fn try_from(path: String) -> Result<AbsPathBuf, Utf8PathBuf> {
        AbsPathBuf::try_from(Utf8PathBuf::from(path))
    }
}

impl TryFrom<PathBuf> for AbsPathBuf {
    type Error = PathBuf;
    fn try_from(path: PathBuf) -> Result<AbsPathBuf, PathBuf> {
        let utf8 = Utf8PathBuf::from_path_buf(path.clone())?;
        if !utf8.is_absolute() {
            return Err(path);
        }
        Ok(AbsPathBuf(utf8))
    }
}

impl<'a> TryFrom<&'a Path> for &'a AbsPath {
    type Error = &'a Path;
    fn try_from(path: &'a Path) -> Result<Self, &'a Path> {
        let Some(utf8) = Utf8Path::from_path(path) else {
            return Err(path);
        };
        if !utf8.is_absolute() {
            return Err(path);
        }
        Ok(AbsPath::assert(utf8))
    }
}

impl<'a> TryFrom<&'a str> for &'a AbsPath {
    type Error = &'a str;
    fn try_from(path: &'a str) -> Result<&'a AbsPath, &'a str> {
        let utf8_path = Utf8Path::new(path);
        if !utf8_path.is_absolute() {
            return Err(path);
        }
        Ok(AbsPath::assert(utf8_path))
    }
}

impl<P: AsRef<Path> + ?Sized> PartialEq<P> for AbsPathBuf {
    fn eq(&self, other: &P) -> bool {
        self.0.as_std_path() == other.as_ref()
    }
}

impl AbsPathBuf {
    /// # Panics if cwd cannot be accessed
    pub fn current_working_directory() -> AbsPathBuf {
        Self::make_absolute(&std::env::current_dir().unwrap())
    }

    /// Constructs an `AbsPathBuf` from a path-like type by resolving it to an absolute path.
    ///
    /// # Panics
    ///
    /// Panics if the cwd could not be accessed or if the path is not valid utf-8.
    pub fn make_absolute<P: ?Sized + AsRef<Path> + std::fmt::Debug>(path: &P) -> AbsPathBuf {
        path::absolute(path).map(AbsPathBuf::assert_absolute_and_utf8).unwrap().normalize()
    }

    /// Constructs an `AbsPathBuf` from a path-like type by resolving it to an absolute path.
    /// Gives an error if the cwd could not be accessed or if the path is not valid utf-8.
    pub fn try_make_absolute<P: ?Sized + AsRef<Path>>(path: &P) -> Result<AbsPathBuf, &P> {
        let absolute = path::absolute(path).map_err(|_| path)?;
        let utf8 = Utf8PathBuf::from_path_buf(absolute).map_err(|_| path)?;
        let absolute = AbsPathBuf(utf8);
        Ok(absolute)
    }

    /// Wrap the given absolute path in `AbsPathBuf`
    ///
    /// # Panics
    ///
    /// Panics if `path` is not absolute.
    pub fn assert2(path: Utf8PathBuf) -> AbsPathBuf {
        AbsPathBuf::try_from(path)
            .unwrap_or_else(|path| panic!("expected absolute path, got {path}"))
    }

    /// Wrap the given absolute path in `AbsPathBuf`
    ///
    /// # Panics
    ///
    /// Panics if `path` is not absolute.
    pub fn assert(path: impl AsRef<str>) -> AbsPathBuf {
        Self::assert2(path.as_ref().into())
    }

    pub fn new_unchecked(path: Utf8PathBuf) -> AbsPathBuf {
        AbsPathBuf(path)
    }

    /// Wrap the given absolute path in `AbsPathBuf`
    ///
    /// # Panics
    ///
    /// Panics if `path` is not absolute.
    pub fn assert_absolute_and_utf8(path: PathBuf) -> AbsPathBuf {
        AbsPathBuf::assert2(
            Utf8PathBuf::from_path_buf(path)
                .unwrap_or_else(|path| panic!("expected utf8 path, got {}", path.display())),
        )
    }

    /// Coerces to an `AbsPath` slice.
    ///
    /// Equivalent of [`Utf8PathBuf::as_path`] for `AbsPathBuf`.
    pub fn as_path(&self) -> &AbsPath {
        AbsPath::new_unchecked(self.0.as_path())
    }

    /// Equivalent of [`Utf8PathBuf::pop`] for `AbsPathBuf`.
    ///
    /// Note that this won't remove the root component, so `self` will still be
    /// absolute.
    pub fn pop(&mut self) -> bool {
        self.0.pop()
    }

    /// Equivalent of [`PathBuf::push`] for `AbsPathBuf`.
    ///
    /// Extends `self` with `path`.
    ///
    /// If `path` is absolute, it replaces the current path.
    ///
    /// On Windows:
    ///
    /// * if `path` has a root but no prefix (e.g., `\windows`), it
    ///   replaces everything except for the prefix (if any) of `self`.
    /// * if `path` has a prefix but no root, it replaces `self`.
    /// * if `self` has a verbatim prefix (e.g. `\\?\C:\windows`)
    ///   and `path` is not empty, the new path is normalized: all references
    ///   to `.` and `..` are removed.
    pub fn push<P: AsRef<Utf8Path>>(&mut self, suffix: P) {
        self.0.push(suffix)
    }

    pub fn join(&self, path: impl AsRef<Utf8Path>) -> Self {
        Self(self.0.join(path)).normalize()
    }

    pub fn with_extension(&self, extension: &str) -> AbsPathBuf {
        AbsPathBuf::assert2(self.0.with_extension(extension))
    }

    pub fn set_extension(&mut self, extension: &str) -> bool {
        self.0.set_extension(extension)
    }

    pub fn is_file(&self) -> bool {
        self.0.is_file()
    }

    /// Converts a [`AbsPathBuf`] to a [`Utf8PathBuf`].
    ///
    /// This is equivalent to the [`From<AbsPathBuf> for Utf8PathBuf`][from] implementation,
    /// but may aid in type inference.
    ///
    /// [from]: #impl-From<AbsPathBuf>-for-Utf8PathBuf
    ///
    /// # Examples
    ///
    /// ```
    /// use paths::AbsPathBuf;
    /// use camino::Utf8PathBuf;
    ///
    /// let abs_path_buf: AbsPathBuf = "/foo.txt".try_into().unwrap();
    /// let utf8_path_buf = abs_path_buf.into_utf8_path_buf();
    /// assert_eq!(utf8_path_buf.as_str(), "/foo.txt");
    ///
    /// // Convert back to an AbsPathBuf.
    /// let new_abs_path_buf = AbsPathBuf::assert(utf8_path_buf);
    /// assert_eq!(new_abs_path_buf, "/foo.txt");
    /// ```
    #[must_use = "`self` will be dropped if the result is not used"]
    pub fn into_utf8_path_buf(self) -> Utf8PathBuf {
        self.into()
    }

    /// Converts a [`AbsPathBuf`] to a [`PathBuf`].
    ///
    /// This is equivalent to the [`From<AbsPathBuf> for PathBuf`][from] implementation,
    /// but may aid in type inference.
    ///
    /// [from]: #impl-From<AbsPathBuf>-for-PathBuf
    ///
    /// # Examples
    ///
    /// ```
    /// use paths::AbsPathBuf;
    /// use std::path::PathBuf;
    ///
    /// let abs_path_buf: AbsPathBuf = "/foo.txt".try_into().unwrap();
    /// let std_path_buf = abs_path_buf.into_std_path_buf();
    /// assert_eq!(std_path_buf.to_str(), Some("/foo.txt"));
    ///
    /// // Convert back to a AbsPathBuf.
    /// let new_abs_path_buf = AbsPathBuf::assert_absolute_and_utf8(std_path_buf);
    /// assert_eq!(new_abs_path_buf, "/foo.txt");
    /// ```
    #[must_use = "`self` will be dropped if the result is not used"]
    pub fn into_std_path_buf(self) -> PathBuf {
        self.into()
    }
}

impl fmt::Display for AbsPathBuf {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.0, f)
    }
}

/// Wrapper around an absolute [`Utf8Path`].
#[derive(Debug, Ord, PartialOrd, Eq, Hash)]
#[repr(transparent)]
pub struct AbsPath(Utf8Path);

impl<P: AsRef<Path> + ?Sized> PartialEq<P> for AbsPath {
    fn eq(&self, other: &P) -> bool {
        self.0.as_std_path() == other.as_ref()
    }
}

impl AsRef<AbsPath> for AbsPath {
    fn as_ref(&self) -> &AbsPath {
        self
    }
}

impl AsRef<Utf8Path> for AbsPath {
    fn as_ref(&self) -> &Utf8Path {
        &self.0
    }
}

impl AsRef<Path> for AbsPath {
    fn as_ref(&self) -> &Path {
        self.0.as_ref()
    }
}

impl AsRef<OsStr> for AbsPath {
    fn as_ref(&self) -> &OsStr {
        self.0.as_ref()
    }
}

impl ToOwned for AbsPath {
    type Owned = AbsPathBuf;

    fn to_owned(&self) -> Self::Owned {
        AbsPathBuf(self.0.to_owned())
    }
}

impl<'a> TryFrom<&'a Utf8Path> for &'a AbsPath {
    type Error = &'a Utf8Path;
    fn try_from(path: &'a Utf8Path) -> Result<&'a AbsPath, &'a Utf8Path> {
        if !path.is_absolute() {
            return Err(path);
        }
        Ok(AbsPath::assert(path))
    }
}

impl<'a> TryFrom<&'a OsStr> for &'a AbsPath {
    type Error = &'a OsStr;
    fn try_from(path: &'a OsStr) -> Result<&'a AbsPath, &'a OsStr> {
        let utf8path: &Utf8Path = path.try_into().map_err(|_| path)?;
        if !utf8path.is_absolute() {
            return Err(path);
        }
        Ok(AbsPath::assert(utf8path))
    }
}

impl AbsPath {
    /// Creates a new [`AbsPath`] from `path`, assuming that it is absolute.
    pub fn new_unchecked(path: &Utf8Path) -> &AbsPath {
        // SAFETY: This is safe because `path` is a valid reference and repr(transparent).
        unsafe { &*(path as *const Utf8Path as *const AbsPath) }
    }

    pub fn as_utf8_path(&self) -> &Utf8Path {
        &self.0
    }

    /// Wrap the given absolute path in `AbsPath`
    ///
    /// # Panics
    ///
    /// Panics if `path` is not absolute.
    pub fn assert(path: &Utf8Path) -> &AbsPath {
        assert!(path.is_absolute(), "{path} is not absolute");
        unsafe { &*(path as *const Utf8Path as *const AbsPath) }
    }

    /// Wrap the given absolute path in `AbsPath`
    ///
    /// # Panics
    ///
    /// Panics if `path` is not absolute.
    pub fn assert_absolute_and_utf8<P: ?Sized + AsRef<Path>>(path: &P) -> &AbsPath {
        let path = path.as_ref();
        let path: &Utf8Path = path.try_into().unwrap();
        assert!(path.is_absolute(), "{path} is not absolute");
        unsafe { &*(path as *const Utf8Path as *const AbsPath) }
    }

    /// Equivalent of [`Utf8Path::parent`] for `AbsPath`.
    pub fn parent(&self) -> Option<&AbsPath> {
        self.0.parent().map(AbsPath::assert)
    }

    /// Equivalent of [`Utf8Path::join`] for `AbsPath`.
    pub fn join(&self, path: impl AsRef<Utf8Path>) -> AbsPathBuf {
        AbsPathBuf::assert(Utf8Path::join(self.as_ref(), path)).normalize()
    }

    /// Normalize the given path:
    /// - Removes repeated separators: `/a//b` becomes `/a/b`
    /// - Removes occurrences of `.` and resolves `..`.
    /// - Removes trailing slashes: `/a/b/` becomes `/a/b`.
    ///
    /// # Example
    /// ```ignore
    /// # use paths::AbsPathBuf;
    /// let abs_path_buf = AbsPathBuf::assert("/a/../../b/.//c//".into());
    /// let normalized = abs_path_buf.normalize();
    /// assert_eq!(normalized, AbsPathBuf::assert("/b/c".into()));
    /// ```
    pub fn normalize(&self) -> AbsPathBuf {
        AbsPathBuf(normalize_path(&self.0))
    }

    /// Converts an [`AbsPath`] to an owned [`AbsPathBuf`].
    pub fn to_path_buf(&self) -> AbsPathBuf {
        AbsPathBuf::try_from(self.0.to_path_buf()).unwrap()
    }

    #[deprecated(
        note = "We explicitly do not provide canonicalization API, as that is almost always a wrong solution, see #14430"
    )]
    pub fn canonicalize(&self) -> ! {
        panic!(
            "We explicitly do not provide canonicalization API, as that is almost always a wrong solution, see #14430"
        )
    }

    /// Equivalent of [`Utf8Path::strip_prefix`] for `AbsPath`.
    ///
    /// Returns a relative path.
    pub fn strip_prefix(&self, base: &AbsPath) -> Option<&RelPath> {
        self.0.strip_prefix(base).ok().map(RelPath::new_unchecked)
    }
    pub fn starts_with(&self, base: &AbsPath) -> bool {
        self.0.starts_with(&base.0)
    }
    pub fn ends_with(&self, suffix: &RelPath) -> bool {
        self.0.ends_with(&suffix.0)
    }

    pub fn name_and_extension(&self) -> Option<(&str, Option<&str>)> {
        Some((self.file_stem()?, self.extension()))
    }

    // region:delegate-methods

    // Note that we deliberately don't implement `Deref<Target = Utf8Path>` here.
    //
    // The problem with `Utf8Path` is that it directly exposes convenience IO-ing
    // methods. For example, `Utf8Path::exists` delegates to `fs::metadata`.
    //
    // For `AbsPath`, we want to make sure that this is a POD type, and that all
    // IO goes via `fs`. That way, it becomes easier to mock IO when we need it.

    /// Returns the final component of the [`AbsPath`], if there is one.
    ///
    /// If the path is a normal file, this is the file name.
    /// If it is the path of a directory, this is the directory name.
    ///
    /// Returns [`None`] if the path terminates in `..`.
    ///
    /// # Examples
    ///
    /// ```
    /// use paths::AbsPath;
    ///
    /// assert_eq!(Some("bin"), AbsPath::try_from("/usr/bin/").unwrap().file_name());
    /// assert_eq!(Some("foo.txt"), AbsPath::try_from("tmp/foo.txt").unwrap().file_name());
    /// assert_eq!(Some("foo.txt"), AbsPath::try_from("foo.txt/.").unwrap().file_name());
    /// assert_eq!(Some("foo.txt"), AbsPath::try_from("foo.txt/.//").unwrap().file_name());
    /// assert_eq!(None, AbsPath::try_from("/foo.txt/..").unwrap().file_name());
    /// assert_eq!(None, AbsPath::try_from("/").file_name());
    /// ```
    pub fn file_name(&self) -> Option<&str> {
        self.0.file_name()
    }

    pub fn extension(&self) -> Option<&str> {
        self.0.extension()
    }

    pub fn with_extension(&self, extension: impl AsRef<str>) -> AbsPathBuf {
        AbsPathBuf::new_unchecked(self.0.with_extension(extension))
    }

    pub fn file_stem(&self) -> Option<&str> {
        self.0.file_stem()
    }

    pub fn as_os_str(&self) -> &OsStr {
        self.0.as_os_str()
    }

    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }

    #[deprecated(note = "use Display instead")]
    pub fn display(&self) -> ! {
        unimplemented!()
    }

    #[deprecated(note = "use std::fs::metadata().is_ok() instead")]
    pub fn exists(&self) -> ! {
        unimplemented!()
    }

    pub fn components(&self) -> Utf8Components<'_> {
        self.0.components()
    }

    pub fn is_file(&self) -> bool {
        self.0.is_file()
    }

    /// Converts an [`AbsPath`] to a [`Path`].
    ///
    /// This is equivalent to the [`AsRef<Path> for AbsPathBuf`][asref] implementation,
    /// but may aid in type inference.
    ///
    /// [asref]: AbsPathBuf#impl-AsRef<Path>-for-AbsPathBuf
    ///
    /// # Examples
    ///
    /// ```
    /// use paths::AbsPath;
    /// use std::path::Path;
    ///
    /// let abs_path: &AbsPath = "/foo.txt".try_into().unwrap();
    /// let std_path: &Path = abs_path.as_std_path();
    /// assert_eq!(std_path.to_str(), Some("/foo.txt"));
    ///
    /// // Convert back to a AbsPath.
    /// let new_abs_path = AbsPath::assert_absolute_and_utf8(std_path);
    /// assert_eq!(new_abs_path, "/foo.txt");
    /// ```
    pub fn as_std_path(&self) -> &path::Path {
        self.0.as_std_path()
    }
    // endregion:delegate-methods
}

impl fmt::Display for AbsPath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.0, f)
    }
}

/// Wrapper around a relative [`Utf8PathBuf`].
#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct RelPathBuf(Utf8PathBuf);

impl From<RelPathBuf> for Utf8PathBuf {
    fn from(RelPathBuf(path_buf): RelPathBuf) -> Utf8PathBuf {
        path_buf
    }
}

impl ops::Deref for RelPathBuf {
    type Target = RelPath;
    fn deref(&self) -> &RelPath {
        self.as_path()
    }
}

impl AsRef<Utf8Path> for RelPathBuf {
    fn as_ref(&self) -> &Utf8Path {
        self.0.as_path()
    }
}

impl AsRef<Path> for RelPathBuf {
    fn as_ref(&self) -> &Path {
        self.0.as_ref()
    }
}

impl TryFrom<Utf8PathBuf> for RelPathBuf {
    type Error = Utf8PathBuf;
    fn try_from(path_buf: Utf8PathBuf) -> Result<RelPathBuf, Utf8PathBuf> {
        if !path_buf.is_relative() {
            return Err(path_buf);
        }
        Ok(RelPathBuf(path_buf))
    }
}

impl TryFrom<&str> for RelPathBuf {
    type Error = Utf8PathBuf;
    fn try_from(path: &str) -> Result<RelPathBuf, Utf8PathBuf> {
        RelPathBuf::try_from(Utf8PathBuf::from(path))
    }
}

impl RelPathBuf {
    /// Coerces to a `RelPath` slice.
    ///
    /// Equivalent of [`Utf8PathBuf::as_path`] for `RelPathBuf`.
    pub fn as_path(&self) -> &RelPath {
        RelPath::new_unchecked(self.0.as_path())
    }

    /// Wrap the given relative path in `AbsPathBuf`
    ///
    /// # Panics
    ///
    /// Panics if `path` is not relative.
    pub fn assert_relative_and_utf8(path: PathBuf) -> RelPathBuf {
        RelPathBuf::assert(
            Utf8PathBuf::from_path_buf(path)
                .unwrap_or_else(|path| panic!("expected utf8 path, got {}", path.display())),
        )
    }

    /// Wrap the given relative path in `RelPathBuf`.
    ///
    /// # Panics
    ///
    /// Panics if `path` is not relative.
    pub fn assert(path: Utf8PathBuf) -> RelPathBuf {
        RelPathBuf::try_from(path)
            .unwrap_or_else(|path| panic!("expected relative path, got {path}"))
    }

    /// Wrap the given relative path in `RelPathBuf`.
    ///
    /// # Panics
    ///
    /// Panics if `path` is not relative.
    pub fn assert_str(path: &str) -> RelPathBuf {
        RelPathBuf::try_from(path)
            .unwrap_or_else(|path| panic!("expected relative path, got {path}"))
    }
}

impl fmt::Display for RelPathBuf {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.0, f)
    }
}

/// Wrapper around a relative [`Utf8Path`].
#[derive(Debug, Ord, PartialOrd, Eq, PartialEq, Hash)]
#[repr(transparent)]
pub struct RelPath(Utf8Path);

impl AsRef<Utf8Path> for RelPath {
    fn as_ref(&self) -> &Utf8Path {
        &self.0
    }
}

impl AsRef<Path> for RelPath {
    fn as_ref(&self) -> &Path {
        self.0.as_ref()
    }
}

impl RelPath {
    /// Creates a new [`RelPath`] from `path`, without checking if it is relative.
    pub fn new_unchecked(path: &Utf8Path) -> &RelPath {
        // SAFETY: This is safe because `path` is a valid reference and repr(transparent).
        unsafe { &*(path as *const Utf8Path as *const RelPath) }
    }

    /// Creates a new [`RelPath`] from `&str`, without checking if it is relative.
    pub fn new_unchecked_from_str(path: &str) -> &RelPath {
        // SAFETY: This is safe because `path` is a valid reference and repr(transparent).
        unsafe { &*(path as *const str as *const RelPath) }
    }

    /// Equivalent of [`Utf8Path::to_path_buf`] for `RelPath`.
    pub fn to_path_buf(&self) -> RelPathBuf {
        RelPathBuf::try_from(self.0.to_path_buf()).unwrap()
    }

    pub fn as_utf8_path(&self) -> &Utf8Path {
        self.as_ref()
    }

    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

impl fmt::Display for RelPath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.0, f)
    }
}

/// Taken from <https://github.com/rust-lang/cargo/blob/79c769c3d7b4c2cf6a93781575b7f592ef974255/src/cargo/util/paths.rs#L60-L85>
fn normalize_path(path: &Utf8Path) -> Utf8PathBuf {
    let mut components = path.components().peekable();
    let mut ret = if let Some(c @ Utf8Component::Prefix(..)) = components.peek().copied() {
        components.next();
        Utf8PathBuf::from(c.as_str())
    } else {
        Utf8PathBuf::new()
    };

    for component in components {
        match component {
            Utf8Component::Prefix(..) => unreachable!(),
            Utf8Component::RootDir => {
                ret.push(component.as_str());
            }
            Utf8Component::CurDir => {}
            Utf8Component::ParentDir => {
                ret.pop();
            }
            Utf8Component::Normal(c) => {
                ret.push(c);
            }
        }
    }
    ret
}
