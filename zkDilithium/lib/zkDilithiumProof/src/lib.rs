use winterfell::math::{fields::f23201::BaseElement, FieldElement};
use winterfell::StarkProof;
use std::io::Write;

mod starkpf;
mod utils;
use crate::utils::poseidon_23_spec::{
    DIGEST_SIZE as HASH_DIGEST_WIDTH,
};

use crate::starkpf::{N, K};

use std::ffi::CStr;
use std::ptr;
use std::slice;
use std::time::{Duration, Instant};

#[no_mangle]
pub extern "C" fn prove(zBytes: *const libc::c_uint, wBytes: *const libc::c_uint, qwBytes: *const libc::c_uint, ctildeBytes: *const libc::c_uint, mBytes: *const libc::c_uint, commBytes: *const libc::c_uint, comrBytes: *const libc::c_uint, nonceBytes: *const libc::c_uint, out_len: *mut libc::c_int) -> *const libc::c_uchar {

    //For now lets just assume that the input's length is ok
    //Convert from the C bytes to something rust readable
    let mut z: [[BaseElement; N]; K] = [[BaseElement::new(0); N]; K];
    let mut w: [[BaseElement; N]; K] = [[BaseElement::new(0); N]; K];
    let mut qw: [[BaseElement; N]; K] = [[BaseElement::new(0); N]; K];
    let mut ctilde: [BaseElement; HASH_DIGEST_WIDTH] = [BaseElement::new(0); HASH_DIGEST_WIDTH];
    let mut m: [BaseElement; 12] = [BaseElement::new(0); 12];
    let mut comm: [BaseElement; 24] = [BaseElement::new(0); 24];
    let mut com_r: [BaseElement; HASH_DIGEST_WIDTH] = [BaseElement::new(0); HASH_DIGEST_WIDTH];
    let mut nonce: [BaseElement; 12] = [BaseElement::new(0); 12];

    unsafe {
        for i in 0..K {
            for j in 0..N {
                z[i][j] = BaseElement::new(*(zBytes.add(i*N+j)));
                w[i][j] = BaseElement::new(*(wBytes.add(i*N+j)));
                qw[i][j] = BaseElement::new(*(qwBytes.add(i*N+j)));
            }
        }

        for i in 0..HASH_DIGEST_WIDTH {
            ctilde[i] = BaseElement::new(*(ctildeBytes.add(i)));
        }

        for i in 0..12 {
            m[i] = BaseElement::new(*(mBytes.add(i)));
        }

        for i in 0..24 {
            comm[i] = BaseElement::new(*(commBytes.add(i)));
        }

        for i in 0..HASH_DIGEST_WIDTH {
            com_r[i] = BaseElement::new(*(comrBytes.add(i)));
        }

        for i in 0..12 {
            nonce[i] = BaseElement::new(*(nonceBytes.add(i)));
        }
    }
    let proof_bytes = starkpf::prove(z, w, qw, ctilde, m, comm, com_r, nonce).to_bytes();

    let len = proof_bytes.len();

    let ptr = &proof_bytes[0];

    unsafe {
        *out_len = len as i32;
    }

    /* let proof_bytes_box = proof_bytes.into_boxed_slice();
    let proof_ptr = proof_bytes_box.as_ptr(); // Get raw pointer to the first byte

    println!("proof/first {}", proof_bytes_box[0]);
    let len2 = proof_bytes_box.len();
    println!("proof/last {}", proof_bytes_box[len2-1]);
    println!("len2 {}", len2);

    // Prevent Rust from deallocating the data by leaking the boxed slice
    std::mem::forget(proof_bytes_box);  */

    //Convert rust bytes to C
    return ptr
}

#[no_mangle]
pub extern "C" fn verify(proofBytes: *const libc::c_uchar, len: *const libc::c_int, commBytes: *const libc::c_uint, nonceBytes: *const libc::c_uint) -> libc::c_int {

    let proofSlice: &[u8] = unsafe {slice::from_raw_parts(proofBytes, (*len) as usize)};

    let proof = StarkProof::from_bytes(proofSlice).unwrap();
    let mut comm: [BaseElement; 24] = [BaseElement::new(0); 24];
    let mut nonce: [BaseElement; 12] = [BaseElement::new(0); 12];
    unsafe {
        for i in 0..24 {
            comm[i] = BaseElement::new(*(commBytes.add(i)));
        }

        for i in 0..12 {
            nonce[i] = BaseElement::new(*(nonceBytes.add(i)));
        }
    }

    match starkpf::verify(proof.clone(), comm, nonce) {
        Ok(_) => {
            println!("Verified.");
            return 1;
        },
        Err(msg) => 
        {
            println!("Failed to verify proof: {}", msg);
            return 0;
        }
    }
}

#[cfg(test)]
pub mod test {

    use std::ffi::CString;
    use super::*;
    use libc::c_uchar;
    use libc::c_uint;

    // This is meant to do the same stuff as the main function in the .go files
    #[test]
    fn simulated_main_function () {
        let zbytes: [u32; N*4] = [
            7236416, 7322400, 40383, 7281512, 7325793, 24025, 7326211, 5291, 117106, 112713, 20863, 6187, 7213778, 7228760, 58812, 87860, 125490, 7258386, 109965, 7282435, 7238068, 7253605, 7289636, 98894, 7293137, 7283939, 7318108, 38172, 62975, 7220419, 7259891, 7302142, 7235864, 7276932, 29241, 7235806, 7238997, 7325293, 82949, 20064, 110484, 74108, 104395, 7288714, 44577, 43932, 7331604, 77104, 102302, 7330885, 7262103, 84368, 117071, 7246209, 49871, 72146, 35278, 78481, 98958, 7289570, 74878, 7288891, 7232605, 7306454, 7339551, 128839, 73966, 90904, 7300797, 99222, 67139, 2460, 77206, 7289140, 80260, 120941, 7329908, 7338393, 7279929, 7303032, 32078, 8812, 72767, 29940, 7335449, 7333821, 27135, 7251618, 7312838, 119826, 25426, 7278460, 7298564, 7244263, 7220222, 7295045, 35456, 59409, 11146, 47522, 42061, 86902, 7288060, 18368, 129848, 7043, 125202, 45794, 107292, 90876, 7270220, 7231736, 120618, 20253, 7297740, 7310020, 7309550, 59611, 23287, 102524, 4567, 7304769, 83176, 59236, 10371, 7252328, 24197, 120965, 7314584, 7212625, 7263862, 7237786, 22732, 7209309, 7230287, 108706, 7217324, 114175, 74249, 7251512, 106668, 105118, 7321540, 7262485, 7288342, 7318280, 7316143, 103323, 95299, 7255902, 7225189, 7233023, 7308290, 7210781, 5763, 7326376, 7293574, 7214886, 34706, 7288078, 7325413, 5881, 73455, 75200, 86010, 7304616, 119407, 34371, 118006, 7311340, 7232710, 7295481, 7245288, 34390, 91514, 7232690, 62417, 7335503, 7265333, 86908, 110536, 21413, 93801, 26311, 49672, 32568, 7221305, 92648, 1380, 7286879, 7288849, 7323856, 7297560, 7312684, 7299147, 7218483, 18851, 109152, 83220, 7267770, 64245, 19642, 7294442, 70375, 7229775, 62159, 7311156, 7327262, 51315, 26009, 11259, 7283649, 103015, 7292440, 108086, 30451, 47955, 33842, 30694, 7107, 86948, 7271675, 39567, 7308400, 13885, 7221657, 7293776, 7324253, 7285779, 7259211, 67887, 45261, 5071, 46427, 108114, 7224463, 71834, 7288411, 7217628, 7277888, 69450, 7226905, 7321318, 32804, 7337791, 7301899, 7291244, 7298169, 58595, 7271087, 7283334, 7225956, 7215193, 78192, 29711, 35709,
            7233080, 69230, 7300818, 7279853, 7304481, 33944, 7325327, 9121, 64706, 59848, 81964, 38728, 103566, 7283006, 119164, 79164, 7224035, 7265437, 7256217, 7270629, 7213654, 8855, 4125, 95601, 7265628, 78222, 7308257, 112643, 5570, 7246731, 7314954, 7320204, 7213250, 61643, 7325852, 86303, 99801, 7314976, 7231871, 51234, 35110, 81315, 7288646, 49613, 52010, 7268220, 117504, 7252108, 56736, 81093, 106531, 7306551, 91845, 34033, 3181, 89648, 4157, 54622, 7269129, 51004, 127616, 72088, 7270156, 88292, 128100, 38838, 7259515, 64887, 7267399, 7276461, 69592, 126782, 7314140, 7290131, 29843, 98918, 33673, 7286230, 91830, 74397, 7304587, 7246683, 25707, 7262916, 61265, 7223946, 82942, 7226673, 63246, 7284299, 43607, 7215201, 122332, 49944, 7289184, 32022, 7286168, 110609, 122581, 14189, 7235357, 30380, 128467, 37972, 7228562, 35608, 91585, 7237945, 7332999, 11719, 128499, 7238551, 26752, 30349, 56766, 7263450, 28634, 7220632, 7266294, 7215803, 93242, 70870, 7269775, 103923, 19104, 7226264, 103421, 17136, 7297088, 7288920, 4389, 88500, 90680, 7269281, 7230179, 7247184, 75062, 94464, 7327006, 47226, 58806, 7265337, 7242728, 7322838, 7324151, 88687, 7247977, 57826, 120052, 109478, 7238372, 84059, 11275, 85398, 40499, 102984, 7254894, 42861, 7285388, 9443, 103256, 8851, 60995, 79865, 51418, 7333514, 7327989, 3801, 7320061, 66274, 55159, 16659, 7297248, 718, 24454, 58104, 111031, 7288460, 108932, 31231, 37111, 7320493, 7230181, 7216603, 121297, 7219954, 7286870, 7210292, 7238468, 7283786, 7283347, 84322, 7310338, 92723, 87049, 7213852, 31708, 70986, 7279488, 72108, 7227359, 7234131, 79338, 22094, 95344, 2075, 7223313, 96572, 7330109, 44490, 7252087, 87513, 7240247, 66555, 31943, 7335660, 7300892, 124861, 7265785, 34384, 7263417, 98305, 7296810, 82551, 7331832, 123307, 18413, 7325955, 127281, 7276813, 65693, 7330564, 7332171, 118149, 7337269, 8829, 4384, 7323510, 100646, 7337064, 7317183, 7246445, 20684, 7332600, 48422, 64736, 7313727, 7271420, 7292018, 82612, 58808, 97690, 7247263, 1716, 7337433, 51476,
            7272619, 7271964, 36202, 7251411, 115121, 7327472, 20944, 97923, 7291389, 19318, 126223, 7265137, 7267720, 7330518, 7250478, 104989, 27852, 4669, 7241576, 52415, 45027, 79672, 7077, 80084, 7245578, 7235131, 7248713, 7339222, 3062, 15225, 7262502, 7214610, 41263, 80495, 13772, 95711, 64998, 4304, 7247098, 96720, 102531, 7292707, 11248, 7241379, 7274593, 7332714, 7329108, 81975, 7318112, 7235002, 112627, 1891, 7243439, 79349, 128363, 7291910, 123423, 7294985, 7285398, 37421, 89719, 7222935, 103775, 7228111, 7280317, 108046, 7277953, 119316, 7292569, 7283215, 7252277, 116947, 7305795, 7323230, 7234922, 23237, 57782, 92025, 126360, 7308748, 7310079, 123769, 43561, 24920, 93807, 63866, 81245, 60171, 21484, 7305608, 7328618, 85710, 7287297, 54557, 76025, 47123, 7235365, 7297084, 7266233, 7308047, 7258108, 7265072, 7303384, 7334028, 128634, 7221696, 7297416, 7235340, 7250812, 7227369, 94593, 46829, 83446, 7332811, 128887, 7217075, 122110, 20314, 103243, 13326, 7265977, 74766, 7257335, 7317871, 117562, 68584, 23871, 38987, 7245328, 16195, 24260, 99083, 7266616, 7304616, 7210793, 123359, 19824, 7297715, 10103, 103728, 83691, 7234373, 102703, 128622, 16255, 32631, 92645, 7222750, 88668, 4944, 7328723, 7266074, 34985, 7328312, 49962, 6969, 31388, 20293, 7303337, 2478, 20082, 7310730, 99196, 67530, 7240232, 7295789, 7289544, 7288931, 97589, 15444, 128919, 7272143, 7213322, 7300243, 60987, 123904, 90127, 7254692, 7283179, 106669, 7224406, 16639, 43857, 83751, 7240347, 33412, 18771, 7259159, 118515, 7308457, 95254, 7291406, 88383, 7230063, 7296666, 65876, 7266985, 51388, 7252141, 89358, 114164, 11465, 83792, 123752, 7210557, 7272143, 7300716, 66747, 77799, 7309047, 7222895, 127211, 26080, 95300, 16833, 7327515, 7282247, 26047, 82664, 7276815, 7337395, 74569, 53807, 7332998, 126215, 7282829, 7284273, 7307300, 7285861, 7262108, 7261054, 7261721, 7259949, 7269693, 97182, 65162, 7245825, 116109, 7269302, 7287695, 7292556, 14911, 7286469, 7279627, 31718, 7315674, 7306003, 7336717, 7337055, 52291, 93230, 29245, 7272117, 7265450, 7290932, 79329,
            7235000, 96658, 68864, 7305825, 7235859, 32620, 7230738, 102668, 7288573, 7269528, 7324945, 69858, 94770, 57634, 7289663, 7322932, 82534, 52470, 98860, 36042, 111677, 7264564, 7262649, 17352, 14944, 18989, 7244276, 7284921, 7216740, 92331, 63684, 7290335, 108602, 41488, 7330546, 7339457, 71924, 7323268, 14526, 80020, 94322, 7326089, 7279613, 109929, 7212168, 7290390, 7256112, 1434, 7329618, 69353, 115905, 7306149, 7334274, 7209691, 57949, 3744, 7217121, 7284675, 74993, 120275, 46043, 6479, 7213197, 7276665, 7810, 115060, 7227906, 7294981, 7322685, 7228905, 61799, 33457, 67392, 7306903, 128638, 120420, 52972, 64768, 72637, 7259482, 119629, 29832, 64026, 7228209, 122159, 113397, 7317180, 7277378, 98859, 126497, 10594, 119429, 107820, 7236201, 81391, 7338959, 7294455, 103692, 101103, 7268206, 44669, 21644, 13590, 7289055, 74785, 98858, 60489, 7250792, 115180, 33323, 114712, 45857, 7235088, 7332735, 7227689, 7284954, 34341, 7228398, 8234, 104544, 7264647, 24444, 130543, 7218934, 130606, 7231133, 79231, 19700, 15778, 7245177, 7329158, 24985, 7339892, 7301631, 7300667, 7228899, 61261, 3317, 7275591, 7225233, 7232148, 108630, 7338721, 62449, 7281001, 106185, 102261, 11468, 55137, 7304659, 31082, 65615, 7248164, 7335604, 7246340, 7332270, 6537, 37426, 94243, 7230151, 7275566, 7337400, 72210, 7234551, 33529, 7296866, 33682, 84712, 7221657, 7243013, 7277739, 130860, 7266432, 7275621, 7214337, 77690, 96683, 81472, 7287181, 81144, 7304489, 83612, 7323592, 7235071, 7334794, 108110, 94409, 74486, 7214410, 7279569, 121425, 101652, 102663, 7247766, 129760, 7228574, 35076, 73654, 10839, 7276974, 7313006, 56769, 7255626, 7270461, 80230, 7222244, 16248, 7218844, 1676, 15486, 1687, 32058, 7225008, 82934, 21966, 7327824, 7253518, 7270604, 64854, 7323133, 46075, 9917, 87705, 7212499, 7301144, 35359, 15620, 44354, 79901, 7329579, 7315519, 7319013, 7320724, 129877, 7317662, 7317295, 35935, 7238603, 7302447, 7320641, 7309347, 7272933, 7290873, 42740, 54594, 22602, 7239061, 7277996, 7224027, 91428, 117887, 7231172, 7323447, 37007, 128225, 10339
        ];

        let wbytes: [u32; N*4] = [
            5081844, 5507875, 5654359, 4510688, 6198488, 2582286, 1773003, 3634553, 4207894, 7046534, 4160053, 5414056, 921657, 942126, 5425201, 3397440, 58241, 4204217, 3343563, 3994571, 1130633, 6662902, 4361230, 7104021, 1043106, 52624, 5406828, 2358429, 5981593, 23721, 5520226, 1376877, 645188, 589122, 1030664, 394446, 5799413, 6819688, 7027141, 2833072, 134967, 534508, 1311343, 1538969, 950283, 2925009, 841463, 6347441, 5480950, 1421540, 4157804, 2316170, 1708833, 5143099, 5157531, 5317844, 3155549, 3707222, 2412982, 5049180, 6635601, 6740749, 5348468, 1510675, 4897293, 1642944, 6145988, 1769494, 2514679, 1590091, 5885290, 6437858, 6626225, 280275, 1292178, 6387662, 1356867, 4545701, 959093, 4744415, 4679750, 284216, 1853039, 4661181, 1701309, 5495063, 5175354, 2102758, 5532215, 548934, 5382286, 2093509, 2709556, 904931, 1342219, 399330, 2079654, 7277982, 5664689, 6329608, 6302504, 6343067, 5246763, 4242338, 5695713, 7088739, 5516684, 2209429, 3278810, 3206075, 2453811, 5049471, 756107, 6902664, 2519500, 5090769, 5453660, 1382130, 180349, 2060845, 7023926, 754049, 4279098, 6245794, 1429919, 5854074, 488876, 3147372, 1269692, 2485088, 4719979, 2052481, 6295191, 5177447, 3511945, 162130, 1304108, 537522, 5570047, 799688, 6073508, 637567, 2972139, 5966490, 3999962, 1301964, 6540033, 2584568, 324624, 1673613, 5896128, 4902725, 1330426, 6127883, 1111368, 1129912, 3468419, 5187292, 4736331, 2558519, 60499, 571864, 5521037, 4611704, 3038252, 327314, 4153724, 5432786, 135032, 2773059, 316211, 1259725, 7060807, 717414, 4272497, 6193975, 6459251, 1516637, 361614, 1070730, 463380, 2525418, 6133321, 4009114, 5903251, 5252922, 2004489, 4319160, 2415291, 5713788, 6544535, 6649191, 4347502, 6345306, 2780527, 2261828, 6738973, 1021311, 4473860, 1073577, 3267112, 5738928, 1287659, 3605888, 30650, 6567941, 5827217, 6473866, 5485184, 6430991, 2672046, 6429665, 5242754, 4126828, 5472221, 5856828, 1901542, 3670871, 3964823, 1433231, 465447, 1489045, 3911581, 2832408, 3352500, 5336026, 5449643, 4620759, 5898883, 7193508, 437961, 1600766, 1395139, 1460219, 1285011, 1056565, 1409800, 738907, 2912535, 1177543, 248418, 4379132, 4721868, 2138467, 6874163, 1914748, 4675157, 3340, 193940, 2096634, 1105170, 174697, 4567955, 2511935, 2912548, 6717697,
            5126070, 3901435, 1924796, 3983050, 4331056, 2848293, 2030738, 7206163, 4750754, 3072569, 5914108, 6151540, 947087, 243049, 1080944, 3123844, 5740981, 1676825, 5975319, 4252290, 6103975, 3173666, 1144677, 5641017, 3946744, 6609895, 3345181, 4535016, 2070625, 5864155, 4827475, 6270174, 4317282, 102016, 3001033, 5057701, 123265, 3582527, 6200757, 6729805, 6313333, 5046154, 7154820, 4211178, 6448465, 4969633, 6876221, 2267759, 931613, 3305471, 3034747, 5963357, 1921709, 2341731, 362861, 4227790, 7204489, 5271900, 5695692, 1242507, 1167878, 5822408, 6121828, 1641437, 3622492, 980460, 326241, 5085784, 68261, 2443886, 1146203, 6803444, 7213354, 5829057, 4057161, 6355874, 6692116, 6374998, 4891476, 475244, 6466044, 2175364, 1987525, 3648539, 3974807, 1725551, 148323, 5568656, 3074070, 4064868, 4262535, 6996208, 5572527, 6898175, 7023897, 315124, 7248094, 3591846, 804618, 3719967, 3080945, 4403745, 5892872, 4965903, 3852267, 6470710, 4331415, 2003363, 263596, 7105113, 3233862, 2973277, 2449213, 5068135, 1834962, 7140051, 628338, 2882721, 3206622, 483628, 3934383, 2316017, 3631965, 6238324, 3978411, 4654245, 4194478, 1243591, 4498468, 3388954, 4234569, 3463201, 3970565, 5925040, 6249148, 3940940, 6902392, 5959240, 5807602, 1423327, 1565927, 1953289, 4444042, 192020, 182694, 3157369, 129668, 650744, 6912725, 234353, 4615950, 110046, 2737122, 7170900, 6311150, 6191638, 4429184, 757285, 7293753, 3512094, 4967853, 5151514, 6446457, 5309579, 6709729, 7212240, 3546282, 3154276, 3299719, 6543745, 4318371, 1000001, 1965292, 946095, 3765894, 6993011, 6918316, 4919325, 4510447, 3457598, 3485521, 2055031, 3016925, 6279898, 3913214, 2659398, 582418, 1060487, 4178791, 6761778, 6280264, 6671046, 3363659, 6658251, 4684558, 4735652, 5160649, 6095549, 1451159, 2123117, 345007, 5821642, 2000894, 6502685, 2552240, 1550340, 1964776, 6514923, 5166902, 2753083, 1210736, 5743792, 760481, 7292776, 2229611, 6529162, 2229914, 5239640, 2069794, 6733313, 6113307, 636247, 5533525, 4011246, 6048620, 2707408, 2132234, 4952078, 3045959, 1404072, 3535966, 130246, 6850983, 6522935, 3443764, 3581236, 1900614, 221219, 6521977, 6784052, 4742240, 2616024, 7101660, 5207889, 7064111, 3151597, 5814971, 262266, 6499959, 4125319, 1991236, 2963701, 4551934, 1725169, 6207994, 4843335,
            5614093, 319420, 6896613, 4675247, 5097255, 6490382, 1013058, 591163, 4755937, 4234929, 6276643, 3906573, 6421579, 2489103, 1988066, 1651734, 2088577, 1479717, 3038903, 6468618, 1173366, 6527904, 3412161, 164159, 1620189, 5114660, 4015300, 2307963, 6109544, 5291210, 4975537, 3394076, 3617766, 1337891, 1159585, 4214448, 5658784, 4062885, 5534545, 2909297, 880547, 4993765, 6544347, 4425829, 2971874, 7166469, 2613512, 3397207, 3247967, 4305382, 12403, 856834, 15917, 204451, 6099379, 5855894, 5437323, 141260, 1443954, 4905889, 3039125, 4636022, 6860366, 6832264, 2844036, 6494563, 3757266, 604574, 3641217, 6393298, 3593325, 902039, 693088, 4368081, 2272073, 84539, 1217656, 2973042, 6863893, 6630535, 3470617, 5013473, 4172054, 107621, 1146924, 510109, 5044039, 2754817, 6168324, 323688, 3352446, 4476790, 1077708, 4028824, 3188125, 3588461, 2464480, 2251176, 2266470, 4255581, 172250, 4917068, 4087429, 4058599, 2851789, 5672722, 6880455, 81134, 4772323, 670848, 2059545, 4247284, 4038148, 5137593, 2434660, 4264499, 5976025, 3276505, 5644708, 3864253, 7161181, 3046973, 5136852, 634563, 2995605, 482457, 277848, 1621972, 4366927, 1610143, 6578123, 1471345, 3321865, 6054646, 2473506, 2819653, 4312920, 5998336, 6591744, 4045141, 6614379, 7301164, 3826051, 6885526, 3214999, 31168, 872234, 1835339, 1261818, 3508748, 5509875, 6647012, 387022, 1347584, 7275608, 2534525, 3216426, 5726121, 533259, 5077386, 2120350, 3237427, 7141006, 589652, 2741730, 977361, 5007333, 6211733, 4656607, 4083496, 4594749, 386068, 554387, 4754537, 3180071, 4669733, 4204267, 6350371, 2486264, 4963239, 4794097, 4773273, 7081774, 5275090, 4939022, 5742153, 4830963, 4442370, 5614321, 6844134, 3113691, 261205, 2342034, 4457675, 2249946, 3326755, 3947396, 7316504, 2848657, 6454474, 4469534, 6497383, 5660621, 2363269, 2969589, 4830362, 2435443, 5451032, 1012607, 4175527, 3466786, 5294246, 4989759, 6173482, 4770698, 5148205, 2504324, 480671, 2821366, 6447361, 2485108, 2053624, 176869, 3231, 5473013, 4044386, 669367, 4501721, 2671668, 6467220, 829756, 2288903, 4602329, 6936770, 4030420, 895166, 1578923, 4683450, 6039951, 7112767, 1159441, 7218546, 1116182, 1520239, 2004433, 3363283, 2044749, 5956577, 711313, 210642, 3693890, 5351009, 2645704, 7146243, 6345822, 5009126,
            2624382, 772817, 6737481, 713898, 1921186, 2037351, 698647, 181622, 2860195, 7180303, 4934030, 4622752, 4402337, 6060625, 7040480, 6907112, 3633957, 6074041, 5788189, 4284003, 2960545, 7032634, 4442791, 2562445, 3406185, 6505704, 4716138, 4596457, 6900287, 1004658, 358231, 2482839, 566766, 7182638, 6110004, 1694967, 5263914, 510735, 4730615, 2752366, 3310507, 5240855, 2024936, 3382726, 6163609, 5256716, 3755705, 5487738, 4952159, 386312, 2132258, 5975987, 5649083, 6253620, 2460451, 1527671, 1607250, 3227323, 2131594, 4710590, 3541674, 7274283, 6992202, 828159, 2469392, 3124995, 5461208, 4431902, 4792265, 1162282, 3746112, 2030390, 4127448, 4132489, 5262610, 3009634, 916086, 1771280, 5183206, 3233844, 6548191, 6312072, 6366747, 6756251, 2862548, 1992120, 2643651, 4188107, 4253299, 3231939, 2997486, 5597092, 290687, 4255935, 6151312, 7141880, 2882621, 5089888, 5018224, 166975, 2693080, 1038522, 6583185, 258972, 5988515, 6755155, 7130919, 3874096, 2561174, 5727132, 3197059, 394541, 5158188, 3400199, 2597109, 5833104, 6413942, 3865169, 6856966, 2727938, 6303527, 7038170, 2549167, 3573508, 2644874, 1278168, 3270854, 1492978, 3748990, 680516, 5718685, 1069179, 951584, 4136133, 6321073, 2269061, 3739975, 6629105, 5188241, 4199389, 2819549, 6039281, 99648, 6245523, 6826370, 4433139, 3977729, 3457889, 5398967, 4395003, 655448, 5007786, 1842693, 1428628, 4695324, 45999, 2003785, 6022996, 659154, 3662277, 6477247, 5150283, 6190558, 3835600, 6102193, 2865025, 1420597, 4048521, 505851, 7145414, 6309958, 3374413, 6758489, 844997, 3510685, 4383377, 6722391, 6719474, 1126228, 3299488, 2298602, 4961289, 5351472, 6028483, 7218691, 1445142, 5786347, 969554, 7011989, 1559647, 4440189, 1640214, 292129, 1314568, 6642778, 1437179, 5655977, 3884211, 1207581, 7040190, 1887999, 7036165, 1435895, 1498354, 6939506, 3140060, 3220239, 4914890, 3625266, 997633, 3514005, 656087, 3874456, 5055735, 2561779, 6230657, 3879387, 4618432, 5395049, 2615995, 3261438, 4090990, 4840584, 5563811, 5600305, 2922562, 6969363, 524761, 7000135, 3627353, 6208147, 2816903, 4851189, 6702930, 6620430, 1670591, 4263333, 6061098, 1834646, 671685, 4340223, 562965, 2471848, 714326, 3353481, 6341246, 990543, 5786114, 7031820, 480777, 2900775, 1732412, 2641222, 6194598, 6897202, 3790399];


        let qwbytes: [u32; N*4] = [978756, 977652, 6965866, 7004371, 3681460, 5601772, 3782522, 4463072, 4390247, 4879898, 91180, 626780, 1247581, 3071249, 6498885, 4352355, 5497649, 427782, 6465507, 3366760, 571981, 273341, 3905200, 2649938, 3786926, 5438762, 1313191, 3585366, 5727934, 5120226, 4603473, 745190, 3803032, 566774, 515170, 770095, 3139947, 3063303, 2305116, 1233585, 853131, 6029832, 424771, 7188764, 1921014, 5687098, 4195506, 3893435, 2485855, 3530783, 4300172, 6244905, 71127, 657238, 1483484, 3829801, 1223169, 3639786, 3041233, 727751, 7096061, 6898861, 3765568, 4634473, 7026171, 6992915, 2966913, 4462442, 2817437, 1349872, 7141722, 931092, 493483, 826991, 6463363, 5649840, 142668, 5061199, 2830376, 4265061, 4504983, 5994637, 2516598, 843237, 3632813, 199754, 2126111, 5664954, 3475294, 3643314, 6878479, 3073124, 5301025, 313485, 5595254, 5437132, 1499020, 5025294, 5992671, 2879139, 2214163, 2681684, 6406451, 6495517, 2425538, 1807615, 3467037, 7153917, 1316673, 5788667, 3734244, 344709, 5569230, 3313520, 358273, 6030791, 6630131, 6791594, 2658597, 4236925, 728624, 447470, 5479907, 1524271, 938486, 1249164, 4940412, 5264339, 6579502, 2743554, 5511084, 1107117, 6426903, 2996405, 3501369, 5013855, 92087, 1193500, 995352, 5213814, 5946504, 6818630, 814994, 2109069, 4454864, 2974615, 3038936, 2480797, 4629451, 3598532, 3507828, 2964367, 715282, 5580824, 840792, 5058473, 5625978, 863072, 4360396, 3598844, 6491429, 3430029, 737550, 6592960, 5994279, 3459291, 1629226, 1737090, 2576371, 3298870, 310062, 5970334, 330836, 4689623, 3172805, 4796202, 880653, 4262543, 3432192, 6392875, 6208208, 4580440, 4639439, 3332411, 2911651, 4124703, 3417370, 4256621, 5911943, 1028213, 4862452, 3075656, 3020374, 577018, 2323920, 5975273, 5229686, 611639, 2731407, 7008064, 2507328, 3117347, 4636430, 4980758, 1054786, 6182982, 3235784, 5077384, 1443919, 5450362, 4884912, 6787767, 5657151, 4706895, 6268786, 6649114, 2442664, 255570, 6538244, 4686193, 6046907, 925392, 6719164, 6741215, 2834439, 997480, 3123312, 597298, 4944805, 3262473, 4124995, 7042151, 2890340, 5483678, 3897099, 2011006, 6472338, 3462037, 2686270, 6911376, 197832, 7091988, 2304463, 4715498, 1597059, 886305, 5661255, 3083664, 4993022, 568057, 5850572, 5465825, 776015, 1159558, 1555881, 0,
        147196, 5738188, 6843668, 4425889, 2848850, 2118405, 3820466, 6050917, 3007809, 1544955, 3414988, 5445033, 3461599, 2752653, 2905227, 5345898, 1854184, 1206514, 4938404, 7092955, 5910157, 6100308, 5830251, 5856689, 2053714, 1533181, 317525, 5201518, 6252300, 1704399, 2683366, 6377235, 722515, 7258914, 4944842, 4287209, 87166, 528479, 4204163, 64782, 712057, 1465579, 4701756, 6124453, 3121972, 1742877, 3939913, 3618105, 4325102, 5895237, 4786440, 1388533, 2870773, 362230, 3739758, 5227003, 2968549, 359901, 603596, 7154499, 4342443, 7241762, 2397480, 3643239, 1211541, 5341728, 5982551, 905763, 1130292, 1686779, 4636664, 4103467, 5785591, 5989993, 7268362, 1395082, 3627566, 4968994, 5069740, 679784, 5755207, 6218379, 1506426, 2429832, 1980459, 657827, 2072931, 5210851, 3777223, 2380319, 858724, 4923979, 2178883, 1596811, 3152857, 741052, 1799854, 5837716, 5662576, 7271136, 6100323, 6149346, 7296522, 1675694, 2146371, 6813863, 1682331, 5068240, 683393, 6312722, 177075, 4444005, 2839375, 4499272, 2105450, 5079355, 4521364, 6249266, 3373188, 2219567, 1102923, 2250624, 7230613, 2610726, 5518646, 5047116, 4683158, 5683781, 5094329, 131373, 704965, 6729833, 989737, 181176, 3765881, 6421883, 3001813, 4925900, 3080486, 6728194, 807549, 918928, 2538371, 5756870, 3070377, 703556, 2420629, 4980235, 2029503, 5771031, 477094, 2504490, 5702271, 5896793, 2771886, 2080799, 4724806, 5243048, 1645280, 4475762, 5650052, 4414105, 2393973, 1545525, 6496837, 4224416, 3645713, 2534, 6566219, 4907362, 229020, 3478551, 6102035, 4423101, 4085124, 6123792, 6019538, 2773368, 2111752, 3232117, 1118529, 5059048, 6024046, 2423950, 819837, 1772171, 4012430, 2986540, 4785889, 6612582, 5194681, 373346, 5907279, 792283, 2011122, 708640, 5984123, 1866202, 4981999, 6715953, 2078048, 6302943, 6955279, 37933, 2526330, 5572306, 5243936, 4312118, 2262246, 3099986, 4162964, 5475608, 4016199, 6025899, 2261343, 196460, 3759651, 3076708, 2180862, 3536558, 1942464, 559780, 4404100, 3187362, 5921447, 947696, 1740980, 2221913, 6567297, 2621075, 5437558, 4591998, 2754877, 2176933, 6429622, 6466944, 2735072, 4431551, 3923437, 3993039, 6124933, 606264, 4364934, 4638700, 3060773, 3356337, 3915024, 1127405, 2723830, 350698, 809067, 2866648, 4236941, 3898585, 5878788, 0,
        3413414, 5765771, 5486784, 3384896, 3293734, 5710751, 1222165, 108339, 791919, 3432240, 4823235, 1338282, 5612091, 6269586, 6438737, 5398285, 6340853, 1598757, 5976100, 2439164, 7234227, 2408616, 3486371, 4705433, 6144262, 3121512, 2072867, 5898436, 3894714, 2101902, 2018530, 1621645, 5013276, 5490170, 2108098, 5404652, 1782803, 1023257, 2328422, 857906, 878686, 2063555, 6603116, 6656455, 5481908, 4961909, 5274876, 2150592, 1260630, 6643308, 2430966, 4716030, 3015536, 2381902, 4689358, 2257450, 3636972, 5510369, 388744, 3596172, 1486380, 1149863, 5180598, 1373354, 5698931, 5014515, 5602819, 1100039, 5909189, 6201856, 2515208, 829238, 1198586, 6395809, 4333110, 2710944, 4612821, 582648, 2100817, 7108996, 5492698, 2966668, 1661046, 4775557, 4517159, 2406311, 3162563, 4282415, 4031395, 1149426, 1386837, 722231, 7258784, 5998179, 4569048, 2870366, 2682695, 433756, 1937695, 1934126, 2795854, 225437, 5851385, 5439852, 3152618, 2404223, 2658475, 3149598, 735898, 6366851, 5909351, 4780001, 4283573, 4106895, 7276026, 2219835, 6529291, 6468478, 5987435, 3371215, 6041534, 3426835, 522414, 7238985, 5536738, 5335972, 95497, 4745278, 5326257, 960757, 416092, 2123520, 6652217, 1497620, 1136368, 4969552, 5296624, 3319040, 7228572, 3846653, 2878619, 300947, 6029389, 2261488, 4173393, 2839667, 7006341, 2872140, 328309, 1073887, 4533385, 7086562, 1042305, 2848495, 6986513, 6906527, 157181, 4245236, 2461828, 2927754, 6381960, 104042, 123925, 139978, 4151670, 319015, 1499206, 1388398, 1697586, 4693625, 5972210, 1067756, 3525261, 34652, 393884, 125177, 7119140, 1002862, 7101836, 4365433, 1069296, 645866, 2676935, 7064480, 2253940, 1644603, 3631883, 3902539, 288506, 5212671, 798353, 1534855, 1069181, 6986850, 4798400, 5481489, 2567034, 3722159, 3421701, 6119803, 4173243, 1576479, 1068360, 6389120, 2368308, 1956804, 2273356, 4175651, 3690330, 2295739, 4388839, 4748344, 2179445, 3004411, 5828529, 6458888, 2735192, 7035264, 3484447, 2058533, 4135876, 1061752, 1064469, 7188858, 7048387, 790826, 6431174, 4462986, 4222435, 3662361, 4308304, 4046113, 3549413, 2335227, 6048412, 1715577, 5541573, 5740434, 596256, 5386649, 3676244, 656534, 1988999, 1698198, 5265334, 1541716, 2619731, 7111389, 1708115, 5117257, 2432892, 6853246, 1765026, 2098382, 2508579, 0,
        6694404, 4649078, 5133711, 6206863, 442347, 5755333, 5713610, 5914521, 4411585, 291085, 1064478, 1745705, 1300309, 1520778, 5688090, 6987237, 6674916, 4139270, 2478481, 3379339, 178915, 3237795, 263864, 2352266, 7186241, 7101092, 3904695, 3064983, 3325564, 770608, 2218244, 4856871, 4495143, 1830262, 3248785, 106611, 768882, 159252, 1652239, 5670363, 3797300, 6078295, 7147024, 5738449, 1995710, 6169556, 494529, 1009791, 4883883, 2611512, 4370338, 5754920, 5418293, 1747672, 6427631, 6785620, 7202052, 915737, 3281422, 2685993, 3231322, 3535013, 5735798, 3535367, 4294317, 4600082, 2237318, 2486865, 4022737, 2982984, 5493176, 1765180, 5832050, 4474217, 3616029, 2899260, 1084229, 393134, 3061009, 622684, 6321853, 3255301, 6283884, 4617483, 5219243, 6531685, 604922, 4595984, 7253882, 2906251, 5144180, 5831316, 7018367, 5509158, 3868919, 4320739, 5273533, 1081186, 5641339, 3226315, 241350, 796395, 3162154, 5056862, 739159, 4414547, 3963687, 1060834, 3320728, 3923445, 6612249, 2506365, 7168960, 2037658, 1592664, 2697708, 745448, 568728, 5419990, 1473325, 5621578, 5739177, 3267528, 212772, 2402167, 5463158, 4911719, 5706338, 6496308, 1909245, 3054373, 4678309, 5015969, 766962, 6353890, 2660282, 288731, 3297166, 4531416, 5317793, 3392446, 4351646, 854046, 7273730, 628652, 5148973, 6393460, 644002, 6635678, 575999, 3719555, 1691690, 1429544, 3393817, 3330303, 5275424, 6811797, 3528475, 6509935, 6830887, 2057599, 5445749, 861391, 874511, 2542958, 846155, 5585662, 4868011, 3600178, 4127072, 5896051, 134813, 710887, 6103438, 2064922, 1039237, 4366291, 4419397, 1301501, 3782982, 716684, 3916642, 3968573, 6726882, 379575, 4757130, 4550556, 1889808, 4867285, 6865492, 1115596, 5495685, 4017789, 3212650, 2757705, 4318311, 7273239, 7134365, 1122017, 6258824, 4128472, 3695913, 6522710, 6519069, 2817044, 2774557, 1727221, 633572, 4156291, 714679, 5656212, 7194439, 594207, 4088332, 851141, 743774, 4237708, 4594810, 27758, 3395289, 819349, 3886720, 889973, 4874942, 1104235, 5632874, 2427917, 4124054, 2317485, 5432482, 5745157, 5124812, 5087078, 1408204, 5814206, 2636197, 2606144, 5214375, 7065443, 5052619, 7037802, 938849, 3697448, 4691999, 7133262, 784503, 832052, 3813027, 4366051, 7193175, 6210780, 6185084, 6079710, 181215, 5829970, 0];

        let ctildebytes: [u32; HASH_DIGEST_WIDTH] = [1815432, 2829463, 2966834, 3447179, 1893397, 446931, 5012873, 704899, 1417889, 1233229, 1730182, 5394147];

        let mbytes: [u32; 12] = [26331, 30185, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];

        let com_rbytes: [u32; HASH_DIGEST_WIDTH] = [0; HASH_DIGEST_WIDTH];

        let mut len: i32 = 0;

        let start = Instant::now();

        let proof_bytes_ptr = prove(zbytes.as_ptr(), wbytes.as_ptr(), qwbytes.as_ptr(), ctildebytes.as_ptr(), mbytes.as_ptr(), com_rbytes.as_ptr(), &mut len);
    
        println!("{}", verify(proof_bytes_ptr, &len, mbytes.as_ptr()));

        println!("{:?}", start.elapsed());
    }
}