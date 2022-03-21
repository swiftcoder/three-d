//!
//! Mid-level modular abstractions of common graphics concepts such as buffer, texture, program, render target and so on.
//! Can be combined with low-level calls in the `context` module as long as any graphics state changes are reset.
//!

mod context;
#[doc(inline)]
pub use context::*;

pub mod buffer;
pub use buffer::*;

pub mod math;
pub use math::*;

pub mod texture;
pub use texture::*;

mod cpu_mesh;
#[doc(inline)]
pub use cpu_mesh::*;

mod cpu_element_buffer;
#[doc(inline)]
pub use cpu_element_buffer::*;

pub mod render_states;
pub use render_states::*;

pub mod render_target;
pub use render_target::*;

mod uniform;
#[doc(inline)]
pub use uniform::*;

mod cpu_material;
#[doc(inline)]
pub use cpu_material::*;

mod cpu_volume;
#[doc(inline)]
pub use cpu_volume::*;

mod camera;
#[doc(inline)]
pub use camera::*;

mod image_effect;
#[doc(inline)]
pub use image_effect::*;

mod image_cube_effect;
#[doc(inline)]
pub use image_cube_effect::*;

mod program;
#[doc(inline)]
pub use program::*;

mod aabb;
#[doc(inline)]
pub use aabb::*;

mod color;
#[doc(inline)]
pub use color::*;

mod viewport;
#[doc(inline)]
pub use viewport::*;

pub use crate::ThreeDResult;
use thiserror::Error;
///
/// Error in the [core](crate::core) module.
///
#[derive(Debug, Error)]
#[allow(missing_docs)]
pub enum CoreError {
    #[error("failed creating context with error: {0}")]
    ContextCreation(String),
    #[error("failed rendering with error: {0}")]
    ContextError(String),
    #[error("failed creating shader: {0}")]
    ShaderCreation(String),
    #[error("failed creating program: {0}")]
    ProgramCreation(String),
    #[error("failed creating buffer: {0}")]
    BufferCreation(String),
    #[error("failed compiling {0} shader: {1}")]
    ShaderCompilation(String, String),
    #[error("failed to link shader program: {0}")]
    ShaderLink(String),
    #[error("the uniform {0} is sent to the shader but not defined or never used")]
    UnusedUniform(String),
    #[error("the attribute {0} is sent to the shader but not defined or never used")]
    UnusedAttribute(String),
    #[error("failed creating a new render target: {0}")]
    RenderTargetCreation(String),
    #[error("cannot copy {0} from a {1} texture")]
    RenderTargetCopy(String, String),
    #[error("cannot read color from anything else but an RGBA texture")]
    ReadWrongFormat,
    #[error("failed creating a new texture: {0}")]
    TextureCreation(String),
    #[error("invalid size of texture data (got {0} pixels but expected {1} pixels)")]
    InvalidTextureLength(usize, usize),
    #[error("the render call requires the {0} vertex buffer which is missing on the given mesh")]
    MissingMeshBuffer(String),
    #[error(
        "if the fragment shader defined 'in vec3 tang' it also needs to define 'in vec3 bitang'"
    )]
    MissingBitangent,
    #[error("{0} buffer length must be {1}, actual length is {2}")]
    InvalidBufferLength(String, usize, usize),
    #[error("mesh must have both normals and uv coordinates to be able to compute tangents")]
    FailedComputingTangents,
    #[error("the number of vertices must be divisable by 3, actual count is {0}")]
    InvalidNumberOfVertices(usize),
    #[error("data for element at index {0} has length {1} but a length of {2} was expected")]
    InvalidUniformBufferElementLength(u32, usize, usize),
    #[error("the index {0} is outside the expected range [0, {1}]")]
    IndexOutOfRange(usize, usize),
    #[error("cannot take as input a negative minimum distance")]
    NegativeDistance,
    #[error("a minimum must be smaller than a maximum")]
    MinimumLargerThanMaximum,
}

mod internal {
    use crate::context::UniformLocation;
    use crate::core::*;

    pub enum UniformType {
        Value,
        Vec2,
        Vec3,
        Vec4,
        Mat2,
        Mat3,
        Mat4,
    }

    pub trait PrimitiveDataType: DataType + Copy {
        fn send_uniform_with_type(
            context: &Context,
            location: &UniformLocation,
            data: &[Self],
            type_: UniformType,
        );
        fn internal_format_with_size(size: u32) -> u32;
    }

    impl PrimitiveDataType for u8 {
        fn internal_format_with_size(size: u32) -> u32 {
            match size {
                1 => crate::context::R8,
                2 => crate::context::RG8,
                3 => crate::context::RGB8,
                4 => crate::context::RGBA8,
                _ => unreachable!(),
            }
        }

        fn send_uniform_with_type(
            context: &Context,
            location: &UniformLocation,
            data: &[Self],
            type_: UniformType,
        ) {
            let data = data.iter().map(|v| *v as u32).collect::<Vec<_>>();
            u32::send_uniform_with_type(context, location, &data, type_)
        }
    }
    impl PrimitiveDataType for u16 {
        fn internal_format_with_size(size: u32) -> u32 {
            match size {
                1 => crate::context::R16UI,
                2 => crate::context::RG16UI,
                3 => crate::context::RGB16UI,
                4 => crate::context::RGBA16UI,
                _ => unreachable!(),
            }
        }

        fn send_uniform_with_type(
            context: &Context,
            location: &UniformLocation,
            data: &[Self],
            type_: UniformType,
        ) {
            let data = data.iter().map(|v| *v as u32).collect::<Vec<_>>();
            u32::send_uniform_with_type(context, location, &data, type_)
        }
    }
    impl PrimitiveDataType for u32 {
        fn internal_format_with_size(size: u32) -> u32 {
            match size {
                1 => crate::context::R32UI,
                2 => crate::context::RG32UI,
                3 => crate::context::RGB32UI,
                4 => crate::context::RGBA32UI,
                _ => unreachable!(),
            }
        }

        fn send_uniform_with_type(
            context: &Context,
            location: &UniformLocation,
            data: &[Self],
            type_: UniformType,
        ) {
            unsafe {
                match type_ {
                    UniformType::Value => context.uniform_1_u32_slice(Some(location), data),
                    UniformType::Vec2 => context.uniform_2_u32_slice(Some(location), data),
                    UniformType::Vec3 => context.uniform_3_u32_slice(Some(location), data),
                    UniformType::Vec4 => context.uniform_4_u32_slice(Some(location), data),
                    _ => unimplemented!(),
                }
            }
        }
    }
    impl PrimitiveDataType for i8 {
        fn internal_format_with_size(size: u32) -> u32 {
            match size {
                1 => crate::context::R8I,
                2 => crate::context::RG8I,
                3 => crate::context::RGB8I,
                4 => crate::context::RGBA8I,
                _ => unreachable!(),
            }
        }

        fn send_uniform_with_type(
            context: &Context,
            location: &UniformLocation,
            data: &[Self],
            type_: UniformType,
        ) {
            let data = data.iter().map(|v| *v as i32).collect::<Vec<_>>();
            i32::send_uniform_with_type(context, location, &data, type_)
        }
    }
    impl PrimitiveDataType for i16 {
        fn internal_format_with_size(size: u32) -> u32 {
            match size {
                1 => crate::context::R16I,
                2 => crate::context::RG16I,
                3 => crate::context::RGB16I,
                4 => crate::context::RGBA16I,
                _ => unreachable!(),
            }
        }

        fn send_uniform_with_type(
            context: &Context,
            location: &UniformLocation,
            data: &[Self],
            type_: UniformType,
        ) {
            let data = data.iter().map(|v| *v as i32).collect::<Vec<_>>();
            i32::send_uniform_with_type(context, location, &data, type_)
        }
    }
    impl PrimitiveDataType for i32 {
        fn internal_format_with_size(size: u32) -> u32 {
            match size {
                1 => crate::context::R32I,
                2 => crate::context::RG32I,
                3 => crate::context::RGB32I,
                4 => crate::context::RGBA32I,
                _ => unreachable!(),
            }
        }

        fn send_uniform_with_type(
            context: &Context,
            location: &UniformLocation,
            data: &[Self],
            type_: UniformType,
        ) {
            unsafe {
                match type_ {
                    UniformType::Value => context.uniform_1_i32_slice(Some(location), data),
                    UniformType::Vec2 => context.uniform_2_i32_slice(Some(location), data),
                    UniformType::Vec3 => context.uniform_3_i32_slice(Some(location), data),
                    UniformType::Vec4 => context.uniform_4_i32_slice(Some(location), data),
                    _ => unimplemented!(),
                }
            }
        }
    }
    impl PrimitiveDataType for f16 {
        fn internal_format_with_size(size: u32) -> u32 {
            match size {
                1 => crate::context::R16F,
                2 => crate::context::RG16F,
                3 => crate::context::RGB16F,
                4 => crate::context::RGBA16F,
                _ => unreachable!(),
            }
        }

        fn send_uniform_with_type(
            context: &Context,
            location: &UniformLocation,
            data: &[Self],
            type_: UniformType,
        ) {
            let data = data.iter().map(|v| v.to_f32()).collect::<Vec<_>>();
            f32::send_uniform_with_type(context, location, &data, type_)
        }
    }
    impl PrimitiveDataType for f32 {
        fn internal_format_with_size(size: u32) -> u32 {
            match size {
                1 => crate::context::R32F,
                2 => crate::context::RG32F,
                3 => crate::context::RGB32F,
                4 => crate::context::RGBA32F,
                _ => unreachable!(),
            }
        }

        fn send_uniform_with_type(
            context: &Context,
            location: &UniformLocation,
            data: &[Self],
            type_: UniformType,
        ) {
            unsafe {
                match type_ {
                    UniformType::Value => context.uniform_1_f32_slice(Some(location), data),
                    UniformType::Vec2 => context.uniform_2_f32_slice(Some(location), data),
                    UniformType::Vec3 => context.uniform_3_f32_slice(Some(location), data),
                    UniformType::Vec4 => context.uniform_4_f32_slice(Some(location), data),
                    UniformType::Mat2 => {
                        context.uniform_matrix_2_f32_slice(Some(location), false, data)
                    }
                    UniformType::Mat3 => {
                        context.uniform_matrix_3_f32_slice(Some(location), false, data)
                    }
                    UniformType::Mat4 => {
                        context.uniform_matrix_4_f32_slice(Some(location), false, data)
                    }
                }
            }
        }
    }

    pub trait DataType: std::fmt::Debug + Clone {
        fn internal_format() -> u32;
        fn data_type() -> u32;
        fn size() -> u32;
        fn send_uniform(context: &Context, location: &UniformLocation, data: &[Self]);
    }

    impl<T: DataType + ?Sized> DataType for &T {
        fn internal_format() -> u32 {
            T::internal_format()
        }
        fn data_type() -> u32 {
            T::data_type()
        }
        fn size() -> u32 {
            T::size()
        }

        fn send_uniform(context: &Context, location: &UniformLocation, data: &[Self]) {
            T::send_uniform(
                context,
                location,
                &data.iter().map(|v| (*v).clone()).collect::<Vec<_>>(),
            )
        }
    }

    impl DataType for u8 {
        fn internal_format() -> u32 {
            Self::internal_format_with_size(1)
        }

        fn data_type() -> u32 {
            crate::context::UNSIGNED_BYTE
        }

        fn size() -> u32 {
            1
        }

        fn send_uniform(context: &Context, location: &UniformLocation, data: &[Self]) {
            Self::send_uniform_with_type(context, location, data, UniformType::Value)
        }
    }

    impl DataType for u16 {
        fn internal_format() -> u32 {
            Self::internal_format_with_size(1)
        }
        fn data_type() -> u32 {
            crate::context::UNSIGNED_SHORT
        }

        fn size() -> u32 {
            1
        }

        fn send_uniform(context: &Context, location: &UniformLocation, data: &[Self]) {
            Self::send_uniform_with_type(context, location, data, UniformType::Value)
        }
    }

    impl DataType for u32 {
        fn internal_format() -> u32 {
            Self::internal_format_with_size(1)
        }

        fn data_type() -> u32 {
            crate::context::UNSIGNED_INT
        }

        fn size() -> u32 {
            1
        }

        fn send_uniform(context: &Context, location: &UniformLocation, data: &[Self]) {
            Self::send_uniform_with_type(context, location, data, UniformType::Value)
        }
    }

    impl DataType for i8 {
        fn internal_format() -> u32 {
            Self::internal_format_with_size(1)
        }

        fn data_type() -> u32 {
            crate::context::BYTE
        }

        fn size() -> u32 {
            1
        }

        fn send_uniform(context: &Context, location: &UniformLocation, data: &[Self]) {
            Self::send_uniform_with_type(context, location, data, UniformType::Value)
        }
    }

    impl DataType for i16 {
        fn internal_format() -> u32 {
            Self::internal_format_with_size(1)
        }

        fn data_type() -> u32 {
            crate::context::SHORT
        }

        fn size() -> u32 {
            1
        }

        fn send_uniform(context: &Context, location: &UniformLocation, data: &[Self]) {
            Self::send_uniform_with_type(context, location, data, UniformType::Value)
        }
    }

    impl DataType for i32 {
        fn internal_format() -> u32 {
            Self::internal_format_with_size(1)
        }

        fn data_type() -> u32 {
            crate::context::INT
        }

        fn size() -> u32 {
            1
        }

        fn send_uniform(context: &Context, location: &UniformLocation, data: &[Self]) {
            Self::send_uniform_with_type(context, location, data, UniformType::Value)
        }
    }

    impl DataType for f16 {
        fn internal_format() -> u32 {
            Self::internal_format_with_size(1)
        }
        fn data_type() -> u32 {
            crate::context::HALF_FLOAT
        }

        fn size() -> u32 {
            1
        }

        fn send_uniform(context: &Context, location: &UniformLocation, data: &[Self]) {
            Self::send_uniform_with_type(context, location, data, UniformType::Value)
        }
    }

    impl DataType for f32 {
        fn internal_format() -> u32 {
            Self::internal_format_with_size(1)
        }

        fn data_type() -> u32 {
            crate::context::FLOAT
        }

        fn size() -> u32 {
            1
        }

        fn send_uniform(context: &Context, location: &UniformLocation, data: &[Self]) {
            Self::send_uniform_with_type(context, location, data, UniformType::Value)
        }
    }

    impl<T: PrimitiveDataType> DataType for Vector2<T> {
        fn internal_format() -> u32 {
            T::internal_format_with_size(Self::size())
        }

        fn data_type() -> u32 {
            T::data_type()
        }

        fn size() -> u32 {
            2
        }

        fn send_uniform(context: &Context, location: &UniformLocation, data: &[Self]) {
            let data = data.iter().flat_map(|v| [v.x, v.y]).collect::<Vec<_>>();
            T::send_uniform_with_type(context, location, &data, UniformType::Vec2)
        }
    }

    impl<T: PrimitiveDataType> DataType for [T; 2] {
        fn internal_format() -> u32 {
            T::internal_format_with_size(Self::size())
        }

        fn data_type() -> u32 {
            T::data_type()
        }

        fn size() -> u32 {
            2
        }

        fn send_uniform(context: &Context, location: &UniformLocation, data: &[Self]) {
            let data = data.iter().flatten().map(|v| *v).collect::<Vec<_>>();
            T::send_uniform_with_type(context, location, &data, UniformType::Vec2)
        }
    }

    impl<T: PrimitiveDataType> DataType for Vector3<T> {
        fn internal_format() -> u32 {
            T::internal_format_with_size(Self::size())
        }
        fn data_type() -> u32 {
            T::data_type()
        }

        fn size() -> u32 {
            3
        }

        fn send_uniform(context: &Context, location: &UniformLocation, data: &[Self]) {
            let data = data
                .iter()
                .flat_map(|v| [v.x, v.y, v.z])
                .collect::<Vec<_>>();
            T::send_uniform_with_type(context, location, &data, UniformType::Vec3)
        }
    }

    impl<T: PrimitiveDataType> DataType for [T; 3] {
        fn internal_format() -> u32 {
            T::internal_format_with_size(Self::size())
        }
        fn data_type() -> u32 {
            T::data_type()
        }

        fn size() -> u32 {
            3
        }

        fn send_uniform(context: &Context, location: &UniformLocation, data: &[Self]) {
            let data = data.iter().flatten().map(|v| *v).collect::<Vec<_>>();
            T::send_uniform_with_type(context, location, &data, UniformType::Vec3)
        }
    }

    impl<T: PrimitiveDataType> DataType for Vector4<T> {
        fn internal_format() -> u32 {
            T::internal_format_with_size(Self::size())
        }

        fn data_type() -> u32 {
            T::data_type()
        }

        fn size() -> u32 {
            4
        }

        fn send_uniform(context: &Context, location: &UniformLocation, data: &[Self]) {
            let data = data
                .iter()
                .flat_map(|v| [v.x, v.y, v.z, v.w])
                .collect::<Vec<_>>();
            T::send_uniform_with_type(context, location, &data, UniformType::Vec4)
        }
    }

    impl<T: PrimitiveDataType> DataType for [T; 4] {
        fn internal_format() -> u32 {
            T::internal_format_with_size(Self::size())
        }

        fn data_type() -> u32 {
            T::data_type()
        }

        fn size() -> u32 {
            4
        }

        fn send_uniform(context: &Context, location: &UniformLocation, data: &[Self]) {
            let data = data.iter().flatten().map(|v| *v).collect::<Vec<_>>();
            T::send_uniform_with_type(context, location, &data, UniformType::Vec4)
        }
    }

    impl<T: PrimitiveDataType> DataType for Quaternion<T> {
        fn internal_format() -> u32 {
            T::internal_format_with_size(Self::size())
        }

        fn data_type() -> u32 {
            T::data_type()
        }

        fn size() -> u32 {
            4
        }

        fn send_uniform(context: &Context, location: &UniformLocation, data: &[Self]) {
            let data = data
                .iter()
                .flat_map(|v| [v.v.x, v.v.y, v.v.z, v.s])
                .collect::<Vec<_>>();
            T::send_uniform_with_type(context, location, &data, UniformType::Vec4)
        }
    }

    impl DataType for Color {
        fn internal_format() -> u32 {
            u8::internal_format_with_size(Self::size())
        }

        fn data_type() -> u32 {
            u8::data_type()
        }

        fn size() -> u32 {
            4
        }

        fn send_uniform(context: &Context, location: &UniformLocation, data: &[Self]) {
            let data = data
                .iter()
                .flat_map(|v| {
                    [
                        v.r as f32 / 255.0,
                        v.g as f32 / 255.0,
                        v.b as f32 / 255.0,
                        v.a as f32 / 255.0,
                    ]
                })
                .collect::<Vec<_>>();
            f32::send_uniform_with_type(context, location, &data, UniformType::Vec4)
        }
    }

    impl<T: PrimitiveDataType> DataType for Matrix2<T> {
        fn internal_format() -> u32 {
            T::internal_format_with_size(Self::size())
        }

        fn data_type() -> u32 {
            T::data_type()
        }

        fn size() -> u32 {
            4
        }

        fn send_uniform(context: &Context, location: &UniformLocation, data: &[Self]) {
            let data = data
                .iter()
                .flat_map(|v| [v.x.x, v.x.y, v.y.x, v.y.y])
                .collect::<Vec<_>>();
            T::send_uniform_with_type(context, location, &data, UniformType::Mat2)
        }
    }

    impl<T: PrimitiveDataType> DataType for Matrix3<T> {
        fn internal_format() -> u32 {
            T::internal_format_with_size(Self::size())
        }

        fn data_type() -> u32 {
            T::data_type()
        }

        fn size() -> u32 {
            9
        }

        fn send_uniform(context: &Context, location: &UniformLocation, data: &[Self]) {
            let data = data
                .iter()
                .flat_map(|v| {
                    [
                        v.x.x, v.x.y, v.x.z, v.y.x, v.y.y, v.y.z, v.z.x, v.z.y, v.z.z,
                    ]
                })
                .collect::<Vec<_>>();
            T::send_uniform_with_type(context, location, &data, UniformType::Mat3)
        }
    }

    impl<T: PrimitiveDataType> DataType for Matrix4<T> {
        fn internal_format() -> u32 {
            T::internal_format_with_size(Self::size())
        }

        fn data_type() -> u32 {
            T::data_type()
        }

        fn size() -> u32 {
            16
        }

        fn send_uniform(context: &Context, location: &UniformLocation, data: &[Self]) {
            let data = data
                .iter()
                .flat_map(|v| {
                    [
                        v.x.x, v.x.y, v.x.z, v.x.w, v.y.x, v.y.y, v.y.z, v.y.w, v.z.x, v.z.y,
                        v.z.z, v.z.w, v.w.x, v.w.y, v.w.z, v.w.w,
                    ]
                })
                .collect::<Vec<_>>();
            T::send_uniform_with_type(context, location, &data, UniformType::Mat4)
        }
    }
}

use internal::DataType;
fn to_byte_slice<'a, T: DataType>(data: &'a [T]) -> &'a [u8] {
    unsafe {
        std::slice::from_raw_parts(
            data.as_ptr() as *const _,
            data.len() * std::mem::size_of::<T>(),
        )
    }
}

fn from_byte_slice<'a, T: DataType>(data: &'a [u8]) -> &'a [T] {
    unsafe {
        let (_prefix, values, _suffix) = data.align_to::<T>();
        values
    }
}

fn format_from_data_type<T: crate::core::internal::DataType>() -> u32 {
    match T::size() {
        1 => crate::context::RED,
        2 => crate::context::RG,
        3 => crate::context::RGB,
        4 => crate::context::RGBA,
        _ => unreachable!(),
    }
}
