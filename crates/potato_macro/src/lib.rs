/// Generic macro for extracting Python objects into Rust enum variants.
/// Returns early with Ok(variant) on first successful match.
///
/// # Usage for early return (like try_extract_message)
/// ```ignore
/// try_extract_py_object!(
///     item,
///     OpenAIChatMessage => MessageNum::OpenAIMessageV1,
///     AnthropicMessage => MessageNum::AnthropicMessageV1,
/// );
/// ```
#[macro_export]
macro_rules! try_extract_py_object {
    ($item:expr, $($py_type:ty => $variant:path),+ $(,)?) => {
        $(
            if $item.is_instance_of::<$py_type>() {
                return Ok($variant($item.extract::<$py_type>()?));
            }
        )+
    };
}

/// Macro for extracting Python objects and pushing to a collection.
/// Works both inside and outside of loops by returning a bool indicating success.
///
/// # Usage in loops
/// ```ignore
/// if extract_and_push!(
///     item => parts,
///     ImageContentPart => |i| ContentPart::ImageUrl(i),
///     InputAudioContentPart => |a| ContentPart::InputAudio(a),
/// ) {
///     continue; // Optional: continue if in a loop
/// }
/// ```
///
/// # Usage outside loops (returns true if matched)
/// ```ignore
/// let matched = extract_and_push!(
///     item => parts,
///     ImageContentPart => |i| ContentPart::ImageUrl(i),
/// );
/// if !matched {
///     // Handle unmatched type
/// }
/// ```
#[macro_export]
macro_rules! extract_and_push {
    ($item:expr => $collection:expr, $($py_type:ty => $transform:expr),+ $(,)?) => {
        {
            let mut matched = false;
            $(
                if !matched && $item.is_instance_of::<$py_type>() {
                    let extracted = $item.extract::<$py_type>()?;
                    $collection.push($transform(extracted));
                    matched = true;
                }
            )+
            matched
        }
    };
}

/// Specialized alias for message extraction (backward compatibility)
#[macro_export]
macro_rules! try_extract_message {
    ($item:expr, $($msg_type:ty => $variant:path),+ $(,)?) => {
        $crate::try_extract_py_object!($item, $($msg_type => $variant),+)
    };
}

/// Macro for extracting Python objects into Rust enum variants.
/// Returns early with Ok(variant) on first successful match.
/// Works similarly to try_extract_py_object but without wrapping in outer variant.
///
/// # Usage
/// ```ignore
/// try_extract_to_enum!(
///     data,
///     PyString => |s: String| DataNum::Text(s),
///     Blob => |b| DataNum::InlineData(b),
///     FileData => |f| DataNum::FileData(f),
/// );
/// ```
#[macro_export]
macro_rules! try_extract_to_enum {
    ($item:expr, $($py_type:ty => $transform:expr),+ $(,)?) => {
        $(
            if $item.is_instance_of::<$py_type>() {
                return Ok($transform($item.extract::<$py_type>()?));
            }
        )+
    };
}
