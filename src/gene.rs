use crate::neural::{ConnectionPacked, NeuralNet, WEIGHT_RANGE};
use std::f32::consts::PI;

pub struct HSLColor {
    pub hue: u16,
    pub saturation: f32,
}

impl HSLColor {
    pub fn to_hex(&self) -> String {
        let (h, s, l) = (self.hue as f32, self.saturation, 0.5);

        // Convert HSL to RGB
        let m: f32 = 2.0 * l - 1.0;
        let c = (1.0 - m.abs()) * s; // Chroma
        let x = c * (1.0 - ((h / 60.0) % 2.0 - 1.0).abs());
        let m = l - c / 2.0;

        let (r1, g1, b1) = match h {
            0.0..=60.0 => (c, x, 0.0),
            60.0..=120.0 => (x, c, 0.0),
            120.0..=180.0 => (0.0, c, x),
            180.0..=240.0 => (0.0, x, c),
            240.0..=300.0 => (x, 0.0, c),
            300.0..=360.0 => (c, 0.0, x),
            _ => (0.0, 0.0, 0.0), // Default case (should not happen)
        };

        // Convert adjusted RGB values to the 0-255 range
        let (r, g, b) = (
            ((r1 + m) * 255.0).round() as u8,
            ((g1 + m) * 255.0).round() as u8,
            ((b1 + m) * 255.0).round() as u8,
        );

        // Format the RGB values as a hex string
        format!("#{:02X}{:02X}{:02X}", r, g, b)
    }
}

#[derive(Debug, Clone)]
pub struct Genome {
    genes: Vec<Gene>,
    //traits are an attempt at summarizing the organisms behaviour(taking direct
    //input and output connections, only the traits are used in coloring)
    traits: Vec<Gene>,
}

impl Genome {
    pub fn new(genes: Vec<Gene>, traits: Vec<Gene>) -> Self {
        Genome { genes, traits }
    }

    pub fn length(&self) -> usize {
        self.genes.len()
    }

    pub fn get_color(&self) -> HSLColor {
        let colors: Vec<HSLColor> = self.genes.iter().map(|gene| gene.get_color()).collect();
        let mut total_weight = 0.0;
        let mut sum_sin = 0.0;
        let mut sum_cos = 0.0;
        let mut weighted_saturation_sum = 0.0;

        for color in colors {
            let weight = color.saturation;
            total_weight += weight;
            let hue_radians = color.hue as f32 * PI / 180.0;

            sum_sin += weight * hue_radians.cos();
            sum_cos += weight * hue_radians.sin();
            weighted_saturation_sum += color.saturation * weight;
        }

        let blended_hue = if total_weight > 0.0 {
            (sum_cos.atan2(sum_sin) * 180.0 / PI).rem_euclid(360.0)
        } else {
            0.0
        };

        let blended_saturation = if total_weight > 0.0 {
            (weighted_saturation_sum / total_weight).clamp(-4.0, 4.0)
        } else {
            0.0
        };

        // Return the resulting blended HSL color
        HSLColor {
            hue: blended_hue as u16,
            saturation: blended_saturation,
        }
    }
}

impl From<NeuralNet> for Genome {
    fn from(net: NeuralNet) -> Self {
        let mut genes: Vec<Gene> = vec![];
        net.for_each_neuron(|_, _, neuron| {
            for connection in &neuron.connections {
                let packed: ConnectionPacked = connection.clone().into();
                let hex = packed.to_hex();
                genes.push(Gene(hex));
            }
        });

        let traits = net
            .get_summary_connections()
            .into_iter()
            .map(|connection| {
                let packed: ConnectionPacked = connection.into();
                let hex = packed.to_hex();
                Gene(hex)
            })
            .collect();
        Genome { genes, traits }
    }
}

#[derive(Debug, Clone)]
pub struct Gene(String);

impl Gene {
    pub fn get_weight(&self) -> f32 {
        let gene_string = &self.0;
        let bytes = hex::decode(gene_string).unwrap();
        let weight = i16::from_le_bytes([bytes[2], bytes[3]]);
        weight as f32 / 1000.0
    }
}

impl Gene {
    pub fn get_color(&self) -> HSLColor {
        if let Ok(hex) = u32::from_str_radix(self.0.as_str(), 16) {
            //get the gene body ie (first 16 bits)
            let gene_body = (hex >> 16) as u16;
            let mut color_angle = gene_body % 360;
            let color_weight = self.get_weight();
            if color_weight < 0.0 {
                color_angle = (color_angle + 180) % 360
            }

            let color = HSLColor {
                hue: color_angle,
                saturation: color_weight.abs() / WEIGHT_RANGE,
            };

            println!("gene - {} {:?}", color.saturation, color.to_hex());

            color
        } else {
            panic!("Failed to parse gene hex string");
        }
    }
}

mod test {

    use super::*;

    #[test]
    fn convert_random_net_to_genome() {
        let mut test_net = NeuralNet::new(vec![2, 3, 2]);
        test_net.init_random_connections(3, (-4.0, 4.0));
        let genome: Genome = test_net.into();
        println!("{:?}", genome);
    }

    #[test]
    fn convert_neural_net_to_genome() {
        let mut test_net = NeuralNet::new(vec![2, 3, 2]);
        test_net.add_connection((0, 0), (1, 1), 1.2);
        test_net.add_connection((0, 1), (1, 1), 2.1);
        test_net.add_connection((1, 1), (2, 0), 1.0);
        let genome: Genome = test_net.into();
        genome.get_color();
    }

    #[test]
    fn get_genome_color() {
        let mut test_net = NeuralNet::new(vec![2, 1, 3]);
        test_net.add_connection((0, 1), (2, 2), -3.11);
        // test_net.add_connection((0, 0), (2, 0), 2.11);
        // test_net.add_connection((0, 0), (2, 1), 2.11);
        // test_net.add_connection((0, 1), (2, 2), 2.11);
        let genome: Genome = test_net.into();
        let color = genome.get_color();
        println!("{:#?}", color.to_hex());
    }

    #[test]
    fn color_random_organisms() {
        for i in 0..10 {
            let mut test_net = NeuralNet::new(vec![10, 1, 1, 10]);
            test_net.init_random_connections(10, (-4.0, 4.0));
            let genome: Genome = test_net.into();
            println!("genome - {:?}\n", genome.get_color().to_hex());
        }
    }
}
