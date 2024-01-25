use image::gif::Encoder;
use num::traits::bounds;
use num::Complex;
use std::process::Output;
use std::{ops::Index, str::FromStr};
use image::ColorType;
use image::png::PNGEncoder;
use std::fs::{File, FileTimes};
use std::env;

///  Tenta determinar se `c` está no conjunto de Mandelbrot, utilizando no maximo `limit` iterações para decidir. 
///    Se `c` não forum membro, retorna `Some(i)`, onde i é o numero de iterações necessárias para `c` deixar o 
///circulo de raio 2 centrado na origem. Se `c` parece ser um menbro (mais precisamente, se chegamos ao limite 
///de iterações sem ser capaz de provar que `c` não é uma menbro) retorna `None` */
fn escape_time(c: Complex<f64>, limit: usize) -> Option<usize> {
    let mut z = Complex {re: 0.0, im: 0.0};
    for i in 0..limit{
        if z.norm_sqr() > 4.0 {
            return Some(i);
        }
        z = z * z + c;
    }
    None
}

///Analise a string `s` como um par de coordenadas, como `"400x600"` ou `"1.0,0.5"`.
/// 
/// Especificadamente, `s` deve ter a forma <left><sep><right>, onde <sep> é 
/// o caractere dado pelo argumento `separato`, e <left> e <right> são 
/// strings que podem ser analisadas por `T::From_str`. `Separator`
/// deve ser um caractere ASCII
/// 
/// Se `s` tiver a forma adequada, retorna `Some<(x,y)>`.
/// Se não analisar corretamente, retorna `None`.
fn parse_pair<T: FromStr>(s: &str, separator: char) -> Option<(T,T)>{
    match s.find(separator) {
        None => None,
        Some(index) =>{
            match (T::from_str(&s[..index]), T::from_str(&s[index + 1..])){
                (Ok(l), Ok(r))=> Some((l,r)),
                _=>None
            }
        }
    }
}

#[test]
fn test_parse_pair(){
    assert_eq!( parse_pair::<i32>("",         ','), None);
    assert_eq!( parse_pair::<i32>("10",       ','), None);
    assert_eq!( parse_pair::<i32>(",10",      ','), None);
    assert_eq!( parse_pair::<i32>("10,20",    ','), Some((10,20)));
    assert_eq!( parse_pair::<i32>("10,20xy",  ','), None);
    assert_eq!( parse_pair::<f64>("0.5x",     'x'), None);
    assert_eq!( parse_pair::<f64>("0.5x1.5",  'x'), Some((0.5, 1.5)));
}

///Analisa um par de números de ponto flutuante separados
/// por uma vírgula como um número complexo
fn parse_complex(s :&str) -> Option<Complex<f64>> {
    match parse_pair(s, ',') {
        Some((re,im)) => Some(Complex{re, im}),
        None => None
        
    }
}

#[test]
fn test_parse_complex(){
    assert_eq!(parse_complex("1.25, -0.0625"),
               Some(Complex{re: 1.25 , im: -0.0625}));
    
    assert_eq!(parse_complex(",-0.0625"),None);
}

///Dada a linha e a coluna de um pixel na imagem de saída,
/// retorna o ponto correspondente no plano complexo
/// 
/// `bounds` é um par que dá a largura e a altura da imagem em pixels
/// `pixel` é um par (coluna, linha) que indica um pixel específico nessa imagem.
/// Os parâmetros `upper_left` e `lower_right`são pontos no plano
/// complexo que designam a área que nossa imagem cobre.
fn pixel_to_point(bounds: (usize,usize),
                  pixel: (usize,usize),
                  upper_left: Complex<f64>,
                  lower_right: Complex<f64>)
                
 -> Complex<f64>
 {
    let (width, height) = (lower_right.re - upper_left.re,
                                     upper_left.im - lower_right.im);
    Complex {
        re: upper_left.re + pixel.0 as f64 * width / bounds.0 as f64,
        im: upper_left.im - pixel.1 as f64 * height / bounds.1 as f64
        // Por que subtração aqui? pixel.1 aumenta à medida que descemos, 
        // mas o componente imaginário aumenta à medida que subimos 

    }    
}

#[test]
fn test_pixel_to_point() {
    assert_eq!(pixel_to_point((100, 200), (25,175), 
                               Complex { re: (-1.0), im: (1.0) },
                               Complex { re: (1.0), im: (-1.0) } ),
               Complex{re: -0.5, im: -0.75});
}


///Renderiza um retângulo do conjunto de Mandelbrot em um buffer de pixels
/// O argumento `bounds` dá a largura e a altura dos `pixels` do buffer,
/// que contém um pixel em tons de cinza por byte.
/// Os argumentos `upper_left` e `lower_right` especificam
/// pontos no plano complexo correspondente aos cantos
/// superios esquerdo |° e inferior direito _| do buffer de pixels
fn render(pixels: &mut [u8],
                 bounds:(usize,usize),
                 upper_left: Complex<f64>,
                 lower_right: Complex<f64>)
{



   assert!(pixels.len() == bounds.0 * bounds.1);

 for row in 0..bounds.1 {
        for column in 0..bounds.0 {
            let point = pixel_to_point(bounds, (column,row),
                                                     upper_left, lower_right);

            pixels[row * bounds.0 + column] = 
                match escape_time(point, 255) {
                    None => 0,
                    Some(count)=> 255 - count as u8
                    
                };                                         
        } 
    
    }
}

///Escreva o buffer `pixels` cujas dimensões são dadas por `bounds`,
/// para o arquivo chamado `filename`
fn write_image(filename: &str, pixels: &[u8], bounds:(usize,usize)) 
    -> Result<(), std::io::Error> 
    {

        let output = File::create(filename)?;
        let encoder = PNGEncoder::new(output);

        encoder.encode(pixels, bounds.0 as u32, bounds.1 as u32,ColorType::Gray(8))?;


    
    
    Ok(())

    
}
fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() != 5 {
        eprintln!("Usage: {} FILE PIXELS UPPERLEFT LOWERRIGHT", args[0]);

        eprintln!("Example: {} mandel.png 1000x750 -1.20,0.35 -1,0.20", args[0]);

        std::process::exit(1);
    }

    let bounds = parse_pair(&args[2], 'x')
        .expect("erros parsing image dimensions");
    let upper_left = parse_complex(&args[3])
        .expect("error parsing upper left corner point");
    let lower_right = parse_complex(&args[4])
        .expect("error parsing lower right corner point");

    let mut pixels = vec![0; bounds.0 * bounds.1];

    
    let threads = 8;
    let row_per_band = bounds.1 / threads + 1;
    let bands: Vec<&mut[u8]> = 
        pixels.chunks_mut(row_per_band * bounds.0).collect();

    crossbeam::scope(|spawner| {
        for(i, band) in bands.into_iter().enumerate(){
            let top = row_per_band * i;
            let height = band.len() / bounds.0;
            let band_bounds = (bounds.0, height);
            let band_upper_left = 
                pixel_to_point(bounds, (0, top), upper_left, lower_right);
            let band_lower_right = 
                pixel_to_point(bounds, (bounds.0, top + height), upper_left, lower_right);
            spawner.spawn(move |_| {
                render(band, band_bounds, band_upper_left, band_lower_right);
            });
        }
    }).unwrap();

    write_image(&args[1], &pixels, bounds)
        .expect("error writing PNG file");
    

}
