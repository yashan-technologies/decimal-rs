// Copyright 2021 CoD Technologies Corp.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Decimal implementation.

use crate::convert::MAX_I128_REPR;
use crate::error::{DecimalConvertError, DecimalFormatError};
use crate::u256::{POWERS_10, ROUNDINGS, U256};
use stack_buf::StackVec;
use std::cmp::Ordering;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::io;

/// Maximum precision of `Decimal`.
pub const MAX_PRECISION: u32 = 38;
/// Maximum binary data size of `Decimal`.
pub const MAX_BINARY_SIZE: usize = 18;
pub const MAX_SCALE: i16 = 130;
pub const MIN_SCALE: i16 = -126;

const SIGN_MASK: u8 = 0x01;
const SCALE_MASK: u8 = 0x02;
const SCALE_SHIFT: u8 = 1;

/// Computes by Taylor series, not accurate values.
static NATURAL_EXP: [Decimal; 291] = [
    // e^0
    unsafe { Decimal::from_raw_parts(1, 0, false) },
    unsafe { Decimal::from_raw_parts(27182818284590452353602874713526624975, 37, false) },
    unsafe { Decimal::from_raw_parts(73890560989306502272304274605750078133, 37, false) },
    unsafe { Decimal::from_raw_parts(20085536923187667740928529654581717900, 36, false) },
    unsafe { Decimal::from_raw_parts(54598150033144239078110261202860878404, 36, false) },
    // e^5
    unsafe { Decimal::from_raw_parts(14841315910257660342111558004055227960, 35, false) },
    unsafe { Decimal::from_raw_parts(40342879349273512260838718054338827962, 35, false) },
    unsafe { Decimal::from_raw_parts(10966331584284585992637202382881214326, 34, false) },
    unsafe { Decimal::from_raw_parts(29809579870417282747435920994528886736, 34, false) },
    unsafe { Decimal::from_raw_parts(81030839275753840077099966894327599646, 34, false) },
    // e^10
    unsafe { Decimal::from_raw_parts(22026465794806716516957900645284244366, 33, false) },
    unsafe { Decimal::from_raw_parts(59874141715197818455326485792257781616, 33, false) },
    unsafe { Decimal::from_raw_parts(16275479141900392080800520489848678316, 32, false) },
    unsafe { Decimal::from_raw_parts(44241339200892050332610277594908828183, 32, false) },
    unsafe { Decimal::from_raw_parts(12026042841647767777492367707678594496, 31, false) },
    // e^15
    unsafe { Decimal::from_raw_parts(32690173724721106393018550460917213156, 31, false) },
    unsafe { Decimal::from_raw_parts(88861105205078726367630237407814503509, 31, false) },
    unsafe { Decimal::from_raw_parts(24154952753575298214775435180385823883, 30, false) },
    unsafe { Decimal::from_raw_parts(65659969137330511138786503259060033570, 30, false) },
    unsafe { Decimal::from_raw_parts(17848230096318726084491003378872270387, 29, false) },
    // e^20
    unsafe { Decimal::from_raw_parts(48516519540979027796910683054154055870, 29, false) },
    unsafe { Decimal::from_raw_parts(13188157344832146972099988837453027850, 28, false) },
    unsafe { Decimal::from_raw_parts(35849128461315915616811599459784206894, 28, false) },
    unsafe { Decimal::from_raw_parts(97448034462489026000346326848229752775, 28, false) },
    unsafe { Decimal::from_raw_parts(26489122129843472294139162152811882340, 27, false) },
    // e^25
    unsafe { Decimal::from_raw_parts(72004899337385872524161351466126157931, 27, false) },
    unsafe { Decimal::from_raw_parts(19572960942883876426977639787609534281, 26, false) },
    unsafe { Decimal::from_raw_parts(53204824060179861668374730434117744164, 26, false) },
    unsafe { Decimal::from_raw_parts(14462570642914751736770474229969288564, 25, false) },
    unsafe { Decimal::from_raw_parts(39313342971440420743886205808435276867, 25, false) },
    // e^30
    unsafe { Decimal::from_raw_parts(10686474581524462146990468650741401654, 24, false) },
    unsafe { Decimal::from_raw_parts(29048849665247425231085682111679825660, 24, false) },
    unsafe { Decimal::from_raw_parts(78962960182680695160978022635108224222, 24, false) },
    unsafe { Decimal::from_raw_parts(21464357978591606462429776153126088037, 23, false) },
    unsafe { Decimal::from_raw_parts(58346174252745488140290273461039101919, 23, false) },
    // e^35
    unsafe { Decimal::from_raw_parts(15860134523134307281296446257746601247, 22, false) },
    unsafe { Decimal::from_raw_parts(43112315471151952271134222928569253911, 22, false) },
    unsafe { Decimal::from_raw_parts(11719142372802611308772939791190194524, 21, false) },
    unsafe { Decimal::from_raw_parts(31855931757113756220328671701298645997, 21, false) },
    unsafe { Decimal::from_raw_parts(86593400423993746953606932719264934249, 21, false) },
    // e^40
    unsafe { Decimal::from_raw_parts(23538526683701998540789991074903480449, 20, false) },
    unsafe { Decimal::from_raw_parts(63984349353005494922266340351557081880, 20, false) },
    unsafe { Decimal::from_raw_parts(17392749415205010473946813036112352260, 19, false) },
    unsafe { Decimal::from_raw_parts(47278394682293465614744575627442803712, 19, false) },
    unsafe { Decimal::from_raw_parts(12851600114359308275809299632143099259, 18, false) },
    // e^45
    unsafe { Decimal::from_raw_parts(34934271057485095348034797233406099546, 18, false) },
    unsafe { Decimal::from_raw_parts(94961194206024488745133649117118323116, 18, false) },
    unsafe { Decimal::from_raw_parts(25813128861900673962328580021527338043, 17, false) },
    unsafe { Decimal::from_raw_parts(70167359120976317386547159988611740555, 17, false) },
    unsafe { Decimal::from_raw_parts(19073465724950996905250998409538484479, 16, false) },
    // e^50
    unsafe { Decimal::from_raw_parts(51847055285870724640874533229334853872, 16, false) },
    unsafe { Decimal::from_raw_parts(14093490824269387964492143312370168789, 15, false) },
    unsafe { Decimal::from_raw_parts(38310080007165768493035695487861993900, 15, false) },
    unsafe { Decimal::from_raw_parts(10413759433029087797183472933493796442, 14, false) },
    unsafe { Decimal::from_raw_parts(28307533032746939004420635480140745403, 14, false) },
    // e^55
    unsafe { Decimal::from_raw_parts(76947852651420171381827455901293939935, 14, false) },
    unsafe { Decimal::from_raw_parts(20916594960129961539070711572146737783, 13, false) },
    unsafe { Decimal::from_raw_parts(56857199993359322226403488206332533049, 13, false) },
    unsafe { Decimal::from_raw_parts(15455389355901039303530766911174620071, 12, false) },
    unsafe { Decimal::from_raw_parts(42012104037905142549565934307191617692, 12, false) },
    // e^60
    unsafe { Decimal::from_raw_parts(11420073898156842836629571831447656295, 11, false) },
    unsafe { Decimal::from_raw_parts(31042979357019199087073421411071003730, 11, false) },
    unsafe { Decimal::from_raw_parts(84383566687414544890733294803731179603, 11, false) },
    unsafe { Decimal::from_raw_parts(22937831594696098790993528402686136005, 10, false) },
    unsafe { Decimal::from_raw_parts(62351490808116168829092387089284697469, 10, false) },
    // e^65
    unsafe { Decimal::from_raw_parts(16948892444103337141417836114371974954, 9, false) },
    unsafe { Decimal::from_raw_parts(46071866343312915426773184428060086892, 9, false) },
    unsafe { Decimal::from_raw_parts(12523631708422137805135219607443657677, 8, false) },
    unsafe { Decimal::from_raw_parts(34042760499317405213769071870043505954, 8, false) },
    unsafe { Decimal::from_raw_parts(92537817255877876002423979166873458740, 8, false) },
    // e^70
    unsafe { Decimal::from_raw_parts(25154386709191670062657811742521129623, 7, false) },
    unsafe { Decimal::from_raw_parts(68376712297627438667558928266777109561, 7, false) },
    unsafe { Decimal::from_raw_parts(18586717452841279803403701812545411949, 6, false) },
    unsafe { Decimal::from_raw_parts(50523936302761041945570383321857646506, 6, false) },
    unsafe { Decimal::from_raw_parts(13733829795401761877841885298085389320, 5, false) },
    // e^75
    unsafe { Decimal::from_raw_parts(37332419967990016402549083172647001445, 5, false) },
    unsafe { Decimal::from_raw_parts(10148003881138887278324617841317169760, 4, false) },
    unsafe { Decimal::from_raw_parts(27585134545231702062864698199026619434, 4, false) },
    unsafe { Decimal::from_raw_parts(74984169969901204346756305912240604567, 4, false) },
    unsafe { Decimal::from_raw_parts(20382810665126687668323137537172632374, 3, false) },
    // e^80
    unsafe { Decimal::from_raw_parts(55406223843935100525711733958316612937, 3, false) },
    unsafe { Decimal::from_raw_parts(15060973145850305483525941301676749817, 2, false) },
    unsafe { Decimal::from_raw_parts(40939969621274546966609142293278290448, 2, false) },
    unsafe { Decimal::from_raw_parts(11128637547917594120870714781839408062, 1, false) },
    unsafe { Decimal::from_raw_parts(30250773222011423382665663964434287432, 1, false) },
    // e^85
    unsafe { Decimal::from_raw_parts(82230127146229135103043280164077746957, 1, false) },
    unsafe { Decimal::from_raw_parts(22352466037347150474430657323327147399, 0, false) },
    unsafe { Decimal::from_raw_parts(60760302250568721495223289381302760758, 0, false) },
    unsafe { Decimal::from_raw_parts(16516362549940018555283297962648587672, -1, false) },
    unsafe { Decimal::from_raw_parts(44896128191743452462842455796453162784, -1, false) },
    // e^90
    unsafe { Decimal::from_raw_parts(12204032943178408020027100351363697548, -2, false) },
    unsafe { Decimal::from_raw_parts(33174000983357426257555161078525919101, -2, false) },
    unsafe { Decimal::from_raw_parts(90176284050342989314009959821709052567, -2, false) },
    unsafe { Decimal::from_raw_parts(24512455429200857855527729431109153420, -3, false) },
    unsafe { Decimal::from_raw_parts(66631762164108958342448140502408732643, -3, false) },
    // e^95
    unsafe { Decimal::from_raw_parts(18112390828890232821937987580988159254, -4, false) },
    unsafe { Decimal::from_raw_parts(49234582860120583997548620591133044956, -4, false) },
    unsafe { Decimal::from_raw_parts(13383347192042695004617364087061150290, -5, false) },
    unsafe { Decimal::from_raw_parts(36379709476088045792877438267601857313, -5, false) },
    unsafe { Decimal::from_raw_parts(98890303193469467705600309671380371021, -5, false) },
    // e^100
    unsafe { Decimal::from_raw_parts(26881171418161354484126255515800135886, -6, false) },
    unsafe { Decimal::from_raw_parts(73070599793680672726476826340615135883, -6, false) },
    unsafe { Decimal::from_raw_parts(19862648361376543258740468906137709930, -7, false) },
    unsafe { Decimal::from_raw_parts(53992276105801688697616842371936818967, -7, false) },
    unsafe { Decimal::from_raw_parts(14676622301554423285107021120870470922, -8, false) },
    // e^105
    unsafe { Decimal::from_raw_parts(39895195705472158507637572787300953989, -8, false) },
    unsafe { Decimal::from_raw_parts(10844638552900230813361001028568739551, -9, false) },
    unsafe { Decimal::from_raw_parts(29478783914555093773878202487079276618, -9, false) },
    unsafe { Decimal::from_raw_parts(80131642640005911410561058362935555141, -9, false) },
    unsafe { Decimal::from_raw_parts(21782038807290206355539393313936824934, -10, false) },
    // e^110
    unsafe { Decimal::from_raw_parts(59209720276646702989552288155880397734, -10, false) },
    unsafe { Decimal::from_raw_parts(16094870669615180549262332993373505801, -11, false) },
    unsafe { Decimal::from_raw_parts(43750394472613410734625746750879389186, -11, false) },
    unsafe { Decimal::from_raw_parts(11892590228282008819681954096389267312, -12, false) },
    unsafe { Decimal::from_raw_parts(32327411910848593114262354205829189194, -12, false) },
    // e^115
    unsafe { Decimal::from_raw_parts(87875016358370231131069738030496383831, -12, false) },
    unsafe { Decimal::from_raw_parts(23886906014249914254626392949441611667, -13, false) },
    unsafe { Decimal::from_raw_parts(64931342556644621362249507087712085619, -13, false) },
    unsafe { Decimal::from_raw_parts(17650168856917655832911782056447182390, -14, false) },
    unsafe { Decimal::from_raw_parts(47978133272993021860034882895011331584, -14, false) },
    // e^120
    unsafe { Decimal::from_raw_parts(13041808783936322797338790280986488115, -15, false) },
    unsafe { Decimal::from_raw_parts(35451311827611664751894074212478186941, -15, false) },
    unsafe { Decimal::from_raw_parts(96366656736032012717638730141942241231, -15, false) },
    unsafe { Decimal::from_raw_parts(26195173187490626761889810253746390880, -16, false) },
    unsafe { Decimal::from_raw_parts(71205863268893377088330680682701942197, -16, false) },
    // e^125
    unsafe { Decimal::from_raw_parts(19355760420357225687206244905274872200, -17, false) },
    unsafe { Decimal::from_raw_parts(52614411826663857451767767041616346183, -17, false) },
    unsafe { Decimal::from_raw_parts(14302079958348104463583671072905261088, -18, false) },
    unsafe { Decimal::from_raw_parts(38877084059945950922226736883574780745, -18, false) },
    unsafe { Decimal::from_raw_parts(10567887114362588125648834960427354587, -19, false) },
    // e^130
    unsafe { Decimal::from_raw_parts(28726495508178319332673332249621538192, -19, false) },
    unsafe { Decimal::from_raw_parts(78086710735191511717214963161789844250, -19, false) },
    unsafe { Decimal::from_raw_parts(21226168683560893890870118295564590878, -20, false) },
    unsafe { Decimal::from_raw_parts(57698708620330031794130831485493325609, -20, false) },
    unsafe { Decimal::from_raw_parts(15684135116819639406725212333317378882, -21, false) },
    // e^135
    unsafe { Decimal::from_raw_parts(42633899483147210448936866880765989362, -21, false) },
    unsafe { Decimal::from_raw_parts(11589095424138854283480495676005460415, -22, false) },
    unsafe { Decimal::from_raw_parts(31502427499714519184111642911336978953, -22, false) },
    unsafe { Decimal::from_raw_parts(85632476224822491931954909086237584537, -22, false) },
    unsafe { Decimal::from_raw_parts(23277320404788620254741750385140984218, -23, false) },
    // e^140
    unsafe { Decimal::from_raw_parts(63274317071555853643430245123511451556, -23, false) },
    unsafe { Decimal::from_raw_parts(17199742630376622641833783925547830056, -24, false) },
    unsafe { Decimal::from_raw_parts(46753747846325154027207734100637066905, -24, false) },
    unsafe { Decimal::from_raw_parts(12708986318302188795555166499146091281, -25, false) },
    unsafe { Decimal::from_raw_parts(34546606567175463231258517866889865270, -25, false) },
    // e^145
    unsafe { Decimal::from_raw_parts(93907412866476978131540504016909901172, -25, false) },
    unsafe { Decimal::from_raw_parts(25526681395254551047668755808654353440, -26, false) },
    unsafe { Decimal::from_raw_parts(69388714177584033016228037440452491187, -26, false) },
    unsafe { Decimal::from_raw_parts(18861808084906520052196148181812219044, -27, false) },
    unsafe { Decimal::from_raw_parts(51271710169083297668258887684658163998, -27, false) },
    // e^150
    unsafe { Decimal::from_raw_parts(13937095806663796973183419371414574787, -28, false) },
    unsafe { Decimal::from_raw_parts(37884954272746958042494750441949388081, -28, false) },
    unsafe { Decimal::from_raw_parts(10298198277160991943993878773913738166, -29, false) },
    unsafe { Decimal::from_raw_parts(27993405242674970683739228910895090969, -29, false) },
    unsafe { Decimal::from_raw_parts(76093964787853542218200718174787272690, -29, false) },
    // e^155
    unsafe { Decimal::from_raw_parts(20684484173822473091270347966282423297, -30, false) },
    unsafe { Decimal::from_raw_parts(56226257460750335807897650819666306371, -30, false) },
    unsafe { Decimal::from_raw_parts(15283881393781745666100414040841103028, -31, false) },
    unsafe { Decimal::from_raw_parts(41545897061040224373905771068319348361, -31, false) },
    unsafe { Decimal::from_raw_parts(11293345702805569478727022021871312858, -32, false) },
    // e^160
    unsafe { Decimal::from_raw_parts(30698496406442424667364570301654957343, -32, false) },
    unsafe { Decimal::from_raw_parts(83447164942647743609658358092023252638, -32, false) },
    unsafe { Decimal::from_raw_parts(22683291210002404713058390312611402982, -33, false) },
    unsafe { Decimal::from_raw_parts(61659578305794325320049670543781654770, -33, false) },
    unsafe { Decimal::from_raw_parts(16760811125908827725861073497722332472, -34, false) },
    // e^165
    unsafe { Decimal::from_raw_parts(45560608313792156880112864411796691453, -34, false) },
    unsafe { Decimal::from_raw_parts(12384657367292132198269856467846840036, -35, false) },
    unsafe { Decimal::from_raw_parts(33664989073201642477955778901752989037, -35, false) },
    unsafe { Decimal::from_raw_parts(91510928052956339360089438336198973142, -35, false) },
    unsafe { Decimal::from_raw_parts(24875249283177429446603994479964329509, -36, false) },
    // e^170
    unsafe { Decimal::from_raw_parts(67617938104850097226297739817614724043, -36, false) },
    unsafe { Decimal::from_raw_parts(18380461242828247026619661332259011810, -37, false) },
    unsafe { Decimal::from_raw_parts(49963273795075782374799992291440821058, -37, false) },
    unsafe { Decimal::from_raw_parts(13581425924747849789093255011954118328, -38, false) },
    unsafe { Decimal::from_raw_parts(36918143295804664423920014322334714971, -38, false) },
    // e^175
    unsafe { Decimal::from_raw_parts(10035391806143294571946733464755740501, -39, false) },
    unsafe { Decimal::from_raw_parts(27279023188106115192557593199527116730, -39, false) },
    unsafe { Decimal::from_raw_parts(74152073030341784283386937576609008214, -39, false) },
    unsafe { Decimal::from_raw_parts(20156623266094612066329318409141309108, -40, false) },
    unsafe { Decimal::from_raw_parts(54791382747319794379865564450966140139, -40, false) },
    // e^180
    unsafe { Decimal::from_raw_parts(14893842007818383595644410230322886973, -41, false) },
    unsafe { Decimal::from_raw_parts(40485660085792693262271426689569678698, -41, false) },
    unsafe { Decimal::from_raw_parts(11005143412437994843280976031210742493, -42, false) },
    unsafe { Decimal::from_raw_parts(29915081357615969207184701601447122427, -42, false) },
    unsafe { Decimal::from_raw_parts(81317622051281434061126712044925707902, -42, false) },
    // e^185
    unsafe { Decimal::from_raw_parts(22104421435549887327561037093210488312, -43, false) },
    unsafe { Decimal::from_raw_parts(60086047116855861250341632178539649714, -43, false) },
    unsafe { Decimal::from_raw_parts(16333081002168329377271943881088378495, -44, false) },
    unsafe { Decimal::from_raw_parts(44397917290943821356155881988414973276, -44, false) },
    unsafe { Decimal::from_raw_parts(12068605179340023095364473314473432497, -45, false) },
    // e^190
    unsafe { Decimal::from_raw_parts(32805870153846701518250084137059135841, -45, false) },
    unsafe { Decimal::from_raw_parts(89175600705988431420770803324912086042, -45, false) },
    unsafe { Decimal::from_raw_parts(24240441494100795852378097352461489720, -46, false) },
    unsafe { Decimal::from_raw_parts(65892351627238821736753930934534639373, -46, false) },
    unsafe { Decimal::from_raw_parts(17911398206275708900431827624144225532, -47, false) },
    // e^195
    unsafe { Decimal::from_raw_parts(48688228266413197067093362018659672146, -47, false) },
    unsafe { Decimal::from_raw_parts(13234832615645703553069383005626040404, -48, false) },
    unsafe { Decimal::from_raw_parts(35976005001806811307586628488491091980, -48, false) },
    unsafe { Decimal::from_raw_parts(97792920656963176027414937748815917871, -48, false) },
    unsafe { Decimal::from_raw_parts(26582871917376019734003283472389741150, -49, false) },
    // e^200
    unsafe { Decimal::from_raw_parts(72259737681257492581774770421893056951, -49, false) },
    unsafe { Decimal::from_raw_parts(19642233186817958656484864137420231201, -50, false) },
    unsafe { Decimal::from_raw_parts(53393125542082459716222599802082679919, -50, false) },
    unsafe { Decimal::from_raw_parts(14513756292567525940523654914390132839, -51, false) },
    unsafe { Decimal::from_raw_parts(39452479992769427900327573211143818566, -51, false) },
    // e^205
    unsafe { Decimal::from_raw_parts(10724295945198918021924451209369968217, -52, false) },
    unsafe { Decimal::from_raw_parts(29151658790851239660496155224556382547, -52, false) },
    unsafe { Decimal::from_raw_parts(79242424360609307491188688802264059684, -52, false) },
    unsafe { Decimal::from_raw_parts(21540324218248465690209815988756000148, -53, false) },
    unsafe { Decimal::from_raw_parts(58552671901581093475081587475320346051, -53, false) },
    // e^210
    unsafe { Decimal::from_raw_parts(15916266403779241591571863407774423364, -54, false) },
    unsafe { Decimal::from_raw_parts(43264897742306309199371472477969207063, -54, false) },
    unsafe { Decimal::from_raw_parts(11760618534305001227335647241278102208, -55, false) },
    unsafe { Decimal::from_raw_parts(31968675653239935348846785115930182070, -55, false) },
    unsafe { Decimal::from_raw_parts(86899870108103213822063274684049309002, -55, false) },
    // e^215
    unsafe { Decimal::from_raw_parts(23621833781030833300746567469515129092, -56, false) },
    unsafe { Decimal::from_raw_parts(64210801521856135516771541362226454717, -56, false) },
    unsafe { Decimal::from_raw_parts(17454305496765194050281862479081601620, -57, false) },
    unsafe { Decimal::from_raw_parts(47445721460229655544587842889161196570, -57, false) },
    unsafe { Decimal::from_raw_parts(12897084248347162974810234147016917437, -58, false) },
    // e^220
    unsafe { Decimal::from_raw_parts(35057909752387477224025060891275483360, -58, false) },
    unsafe { Decimal::from_raw_parts(95297279023672025386355634986304892255, -58, false) },
    unsafe { Decimal::from_raw_parts(25904486187163901031830171287130712546, -59, false) },
    unsafe { Decimal::from_raw_parts(70415694078135969991088372949671264959, -59, false) },
    unsafe { Decimal::from_raw_parts(19140970165092820820108477320064452781, -60, false) },
    // e^225
    unsafe { Decimal::from_raw_parts(52030551378848545923020205358078977737, -60, false) },
    unsafe { Decimal::from_raw_parts(14143370233782872265039837168370554989, -61, false) },
    unsafe { Decimal::from_raw_parts(38445666299660540093457531706674996418, -61, false) },
    unsafe { Decimal::from_raw_parts(10450615608536754863982177507098957249, -62, false) },
    unsafe { Decimal::from_raw_parts(28407718504895927718534013347769901830, -62, false) },
    // e^230
    unsafe { Decimal::from_raw_parts(77220184999838357175621252140277020406, -62, false) },
    unsafe { Decimal::from_raw_parts(20990622567530634724568039312619468559, -63, false) },
    unsafe { Decimal::from_raw_parts(57058427893360872481970148326895352874, -63, false) },
    unsafe { Decimal::from_raw_parts(15510088770296358097556054518881247548, -64, false) },
    unsafe { Decimal::from_raw_parts(42160792462083288741186917596094351517, -64, false) },
    // e^235
    unsafe { Decimal::from_raw_parts(11460491602311409370637865042895610414, -65, false) },
    unsafe { Decimal::from_raw_parts(31152846067770590954201464312400440172, -65, false) },
    unsafe { Decimal::from_raw_parts(84682215370802619418949577677244718361, -65, false) },
    unsafe { Decimal::from_raw_parts(23019012723610800962705119766260408375, -66, false) },
    unsafe { Decimal::from_raw_parts(62572163995658794914917604846876973579, -66, false) },
    // e^240
    unsafe { Decimal::from_raw_parts(17008877635675862685398902860714557440, -67, false) },
    unsafe { Decimal::from_raw_parts(46234922999541146273426274861568776275, -67, false) },
    unsafe { Decimal::from_raw_parts(12567955102985587136353369613287969585, -68, false) },
    unsafe { Decimal::from_raw_parts(34163243977334849966907467619116852824, -68, false) },
    unsafe { Decimal::from_raw_parts(92865325304802240908397570249090596499, -68, false) },
    // e^245
    unsafe { Decimal::from_raw_parts(25243412626998187770632793234418799940, -69, false) },
    unsafe { Decimal::from_raw_parts(68618709832262784296500189663439273040, -69, false) },
    unsafe { Decimal::from_raw_parts(18652499202934394647893057141276968924, -70, false) },
    unsafe { Decimal::from_raw_parts(50702749638683390134216749367456409844, -70, false) },
    unsafe { Decimal::from_raw_parts(13782436299574148088857901819149382333, -71, false) },
    // e^250
    unsafe { Decimal::from_raw_parts(37464546145026732603499548122029201501, -71, false) },
    unsafe { Decimal::from_raw_parts(10183919499749154121311809801154593781, -72, false) },
    unsafe { Decimal::from_raw_parts(27682763318657855929985771603963318292, -72, false) },
    unsafe { Decimal::from_raw_parts(75249552490640263726958791405721841505, -72, false) },
    unsafe { Decimal::from_raw_parts(20454949113498251750794190253329225813, -73, false) },
    // e^255
    unsafe { Decimal::from_raw_parts(55602316477276754174041540473381702051, -73, false) },
    unsafe { Decimal::from_raw_parts(15114276650041035425200896657072865078, -74, false) },
    unsafe { Decimal::from_raw_parts(41084863568109398732746435014199662608, -74, false) },
    unsafe { Decimal::from_raw_parts(11168023806191082975759894188368741636, -75, false) },
    unsafe { Decimal::from_raw_parts(30357836172167242865270564060096681892, -75, false) },
    // e^260
    unsafe { Decimal::from_raw_parts(82521154418138915708209187078469436590, -75, false) },
    unsafe { Decimal::from_raw_parts(22431575451828987090132598854038981998, -76, false) },
    unsafe { Decimal::from_raw_parts(60975343934414732803540925731945597709, -76, false) },
    unsafe { Decimal::from_raw_parts(16574816940096003310288868055969816163, -77, false) },
    unsafe { Decimal::from_raw_parts(45055023698298121117106125112845233389, -77, false) },
    // e^265
    unsafe { Decimal::from_raw_parts(12247225219987543111692123050999620531, -78, false) },
    unsafe { Decimal::from_raw_parts(33291409764537471210498902650647395181, -78, false) },
    unsafe { Decimal::from_raw_parts(90495434206726229847410205869155592671, -78, false) },
    unsafe { Decimal::from_raw_parts(24599209436265500385962442739613565585, -79, false) },
    unsafe { Decimal::from_raw_parts(66867584005058783767836195501715462777, -79, false) },
    // e^270
    unsafe { Decimal::from_raw_parts(18176493851390999782546650445313340672, -80, false) },
    unsafe { Decimal::from_raw_parts(49408832941333720129685111047602318635, -80, false) },
    unsafe { Decimal::from_raw_parts(13430713274979613085859250297613421779, -81, false) },
    unsafe { Decimal::from_raw_parts(36508463838620754258131757683218532187, -81, false) },
    unsafe { Decimal::from_raw_parts(99240293837476957258975386473680449662, -81, false) },
    // e^275
    unsafe { Decimal::from_raw_parts(26976308738934978232765417912571366677, -82, false) },
    unsafe { Decimal::from_raw_parts(73329209843947893397917976493127739665, -82, false) },
    unsafe { Decimal::from_raw_parts(19932945861406369879404057817936726125, -83, false) },
    unsafe { Decimal::from_raw_parts(54183364522718865591003756988762312406, -83, false) },
    unsafe { Decimal::from_raw_parts(14728565518687920080874372478970627032, -84, false) },
    // e^280
    unsafe { Decimal::from_raw_parts(40036392008717845384002607853055449617, -84, false) },
    unsafe { Decimal::from_raw_parts(10883019687436065167926658665346876179, -85, false) },
    unsafe { Decimal::from_raw_parts(29583114655119494191648535413124937628, -85, false) },
    unsafe { Decimal::from_raw_parts(80415242996231796059259460914427322527, -85, false) },
    unsafe { Decimal::from_raw_parts(21859129376777539785144693723458114365, -86, false) },
    // e^285
    unsafe { Decimal::from_raw_parts(59419274170829680786039665041625326132, -86, false) },
    unsafe { Decimal::from_raw_parts(16151833323879222366041833857187834774, -87, false) },
    unsafe { Decimal::from_raw_parts(43905235020600150754042953190395882915, -87, false) },
    unsafe { Decimal::from_raw_parts(11934680253072108439235558933754921818, -88, false) },
    unsafe { Decimal::from_raw_parts(32441824460394911649740723321265334285, -88, false) },
    // e^290
    unsafe { Decimal::from_raw_parts(88186021912749658986094822427733469383, -88, false) },
];

/// Computes by Taylor series, not accurate values.
static NATURAL_EXP_NEG: [Decimal; 9] = [
    // e^-291
    unsafe { Decimal::from_raw_parts(41716298478166806118243377939293045745, 164, false) },
    unsafe { Decimal::from_raw_parts(15346568571889094399003486191226211569, 164, false) },
    unsafe { Decimal::from_raw_parts(56456870701257797059912015304055553681, 165, false) },
    unsafe { Decimal::from_raw_parts(20769322043867093362333818538068856442, 165, false) },
    // e^-295
    unsafe { Decimal::from_raw_parts(76406065870075445735958388880036815267, 166, false) },
    unsafe { Decimal::from_raw_parts(28108220814391766921916452972683068317, 166, false) },
    unsafe { Decimal::from_raw_parts(10340436565521946602575863724595250916, 166, false) },
    unsafe { Decimal::from_raw_parts(38040340251929620404917847776950070293, 167, false) },
    unsafe { Decimal::from_raw_parts(13994259113851392172977837187029463838, 167, false) },
];

pub(crate) type Buf = stack_buf::StackVec<u8, 256>;

/// High precision decimal.
#[derive(Copy, Clone, Debug, Eq)]
#[repr(C, packed(4))]
pub struct Decimal {
    int_val: u128,
    // A positive scale means a negative power of 10
    scale: i16,
    negative: bool,
    _aligned: u8,
}

impl Decimal {
    /// Zero value, i.e. `0`.
    pub const ZERO: Decimal = unsafe { Decimal::from_raw_parts(0, 0, false) };

    /// i.e. `1`.
    pub const ONE: Decimal = unsafe { Decimal::from_raw_parts(1, 0, false) };

    /// i.e. `-1`.
    const MINUS_ONE: Decimal = unsafe { Decimal::from_raw_parts(1, 0, true) };

    /// i.e. `2`.
    const TWO: Decimal = unsafe { Decimal::from_raw_parts(2, 0, false) };

    /// i.e. `0.5`.
    const ZERO_POINT_FIVE: Decimal = unsafe { Decimal::from_raw_parts(5, 1, false) };

    #[inline]
    pub(crate) const unsafe fn from_raw_parts(int_val: u128, scale: i16, negative: bool) -> Decimal {
        Decimal {
            int_val,
            scale,
            negative,
            _aligned: 0,
        }
    }

    /// Creates a `Decimal` from parts without boundary checking.
    ///
    /// # Safety
    /// User have to guarantee that `int_val` has at most 38 tens digits and `scale` ranges from `[-126, 130]`.
    #[inline]
    pub const unsafe fn from_parts_unchecked(int_val: u128, scale: i16, negative: bool) -> Decimal {
        if int_val != 0 {
            Decimal::from_raw_parts(int_val, scale, negative)
        } else {
            Decimal::ZERO
        }
    }

    /// Creates a `Decimal` from parts.
    ///
    /// `int_val` has at most 38 tens digits, `scale` ranges from `[-126, 130]`.
    #[inline]
    pub const fn from_parts(int_val: u128, scale: i16, negative: bool) -> Result<Decimal, DecimalConvertError> {
        if int_val > MAX_I128_REPR as u128 {
            return Err(DecimalConvertError::Overflow);
        }

        if scale >= MAX_SCALE + MAX_PRECISION as i16 || scale < MIN_SCALE {
            return Err(DecimalConvertError::Overflow);
        }

        Ok(unsafe { Decimal::from_parts_unchecked(int_val, scale, negative) })
    }

    /// Consumes the `Decimal`, returning `(int_val, scale, negative)`.
    #[inline]
    pub const fn into_parts(self) -> (u128, i16, bool) {
        (self.int_val, self.scale, self.negative)
    }

    /// Returns the precision, i.e. the count of significant digits in this decimal.
    #[inline]
    pub fn precision(&self) -> u8 {
        U256::from(self.int_val).count_digits() as u8
    }

    #[inline(always)]
    pub(crate) const fn int_val(&self) -> u128 {
        self.int_val
    }

    /// Returns the scale, i.e. the count of decimal digits in the fractional part.
    /// A positive scale means a negative power of 10.
    #[inline(always)]
    pub const fn scale(&self) -> i16 {
        self.scale
    }

    /// Returns `true` if the sign bit of the decimal is negative.
    #[inline(always)]
    pub const fn is_sign_negative(&self) -> bool {
        self.negative
    }

    /// Returns `true` if the sign bit of the decimal is positive.
    #[inline(always)]
    pub const fn is_sign_positive(&self) -> bool {
        !self.negative
    }

    /// Checks if `self` is zero.
    #[inline]
    pub const fn is_zero(&self) -> bool {
        self.int_val == 0
    }

    /// Computes the absolute value of `self`.
    #[inline]
    pub const fn abs(&self) -> Decimal {
        let mut abs_val = *self;
        abs_val.negative = false;
        abs_val
    }

    #[inline]
    pub(crate) fn neg_mut(&mut self) {
        if !self.is_zero() {
            self.negative = !self.negative;
        }
    }

    #[inline]
    fn encode_header(&self) -> [u8; 2] {
        let sign = if self.is_sign_negative() { 1 } else { 0 };

        let (scale_sign, abs_scale) = if self.scale < 0 {
            (0, (-self.scale) as u8)
        } else {
            (1, self.scale as u8)
        };

        let flags = (scale_sign << SCALE_SHIFT) | sign;

        [flags, abs_scale]
    }

    /// Encodes `self` to `writer` as binary bytes.
    /// Returns total size on success, which is not larger than [`MAX_BINARY_SIZE`].
    fn internal_encode<W: io::Write, const COMPACT: bool>(&self, mut writer: W) -> std::io::Result<usize> {
        let int_bytes: [u8; 16] = self.int_val.to_le_bytes();

        let mut id = 15;
        while id > 0 && int_bytes[id] == 0 {
            id -= 1;
        }

        if COMPACT && id < 2 && self.scale == 0 && self.is_sign_positive() {
            return if id == 0 {
                let size = writer.write(&int_bytes[0..1])?;
                debug_assert_eq!(size, 1);
                Ok(1)
            } else {
                let size = writer.write(&int_bytes[0..2])?;
                debug_assert_eq!(size, 2);
                Ok(2)
            };
        }

        let header = self.encode_header();
        writer.write_all(&header)?;
        writer.write_all(&int_bytes[0..=id])?;
        let size = id + 3;

        Ok(size)
    }

    /// Encodes `self` to `writer` as binary bytes.
    /// Returns total size on success, which is not larger than [`MAX_BINARY_SIZE`].
    #[inline]
    pub fn encode<W: io::Write>(&self, writer: W) -> std::io::Result<usize> {
        self.internal_encode::<_, false>(writer)
    }

    /// Encodes `self` to `writer` as binary bytes.
    /// Returns total size on success, which is not larger than [`MAX_BINARY_SIZE`].
    ///
    /// The only different from [`Decimal::encode`] is it will compact encoded bytes
    /// when `self` is zero or small positive integer.
    #[inline]
    pub fn compact_encode<W: io::Write>(&self, writer: W) -> std::io::Result<usize> {
        self.internal_encode::<_, true>(writer)
    }

    /// Decodes a `Decimal` from binary bytes.
    #[inline]
    pub fn decode(bytes: &[u8]) -> Decimal {
        let len = bytes.len();
        assert!(len > 0);

        if len <= 2 {
            let int_val = if len == 1 {
                bytes[0] as u128
            } else {
                ((bytes[1] as u128) << 8) | (bytes[0] as u128)
            };

            return unsafe { Decimal::from_parts_unchecked(int_val, 0, false) };
        }

        let flags = bytes[0];
        let abs_scale = bytes[1];

        let negative = (flags & SIGN_MASK) == 1;
        let scale = if (flags & SCALE_MASK) != 0 {
            abs_scale as i16
        } else {
            -(abs_scale as i16)
        };

        let mut int_bytes = [0; 16];
        if len < MAX_BINARY_SIZE {
            int_bytes[0..len - 2].copy_from_slice(&bytes[2..]);
        } else {
            int_bytes.copy_from_slice(&bytes[2..MAX_BINARY_SIZE]);
        }
        let int = u128::from_le_bytes(int_bytes);

        unsafe { Decimal::from_parts_unchecked(int, scale, negative) }
    }

    /// Computes the smallest integer that is greater than or equal to `self`.
    #[inline]
    pub fn ceil(&self) -> Decimal {
        if self.scale <= 0 {
            return *self;
        }

        if self.scale > MAX_PRECISION as i16 {
            return if self.negative { Decimal::ZERO } else { Decimal::ONE };
        }

        let divisor = POWERS_10[self.scale as usize].low();
        let int_val = self.int_val / divisor;

        let int_val = if !self.negative && self.int_val % divisor != 0 {
            int_val + 1
        } else {
            int_val
        };

        unsafe { Decimal::from_parts_unchecked(int_val, 0, self.negative) }
    }

    /// Computes the largest integer that is equal to or less than `self`.
    #[inline]
    pub fn floor(&self) -> Decimal {
        if self.scale <= 0 {
            return *self;
        }

        if self.scale > MAX_PRECISION as i16 {
            return if self.negative {
                Decimal::MINUS_ONE
            } else {
                Decimal::ZERO
            };
        }

        let divisor = POWERS_10[self.scale as usize].low();
        let int_val = self.int_val / divisor;

        let int_val = if !self.negative || self.int_val % divisor == 0 {
            int_val
        } else {
            int_val + 1
        };

        unsafe { Decimal::from_parts_unchecked(int_val, 0, self.negative) }
    }

    /// Truncate a value to have `scale` digits after the decimal point.
    /// We allow negative `scale`, implying a truncation before the decimal
    /// point.
    #[inline]
    pub fn trunc(&self, scale: i16) -> Decimal {
        // Limit the scale value to avoid possible overflow in calculations
        let real_scale = if !self.is_zero() {
            scale.max(MIN_SCALE).min(MAX_SCALE + MAX_PRECISION as i16 - 1)
        } else {
            return Decimal::ZERO;
        };

        if self.scale <= real_scale {
            return *self;
        }

        let e = self.scale - real_scale;
        debug_assert!(e > 0);
        if e > MAX_PRECISION as i16 {
            return Decimal::ZERO;
        }

        let int_val = self.int_val / POWERS_10[e as usize].low();

        unsafe { Decimal::from_parts_unchecked(int_val, real_scale, self.negative) }
    }

    /// Round a value to have `scale` digits after the decimal point.
    /// We allow negative `scale`, implying rounding before the decimal
    /// point.
    #[inline]
    pub fn round(&self, scale: i16) -> Decimal {
        // Limit the scale value to avoid possible overflow in calculations
        let real_scale = if !self.is_zero() {
            scale.max(MIN_SCALE).min(MAX_SCALE + MAX_PRECISION as i16 - 1)
        } else {
            return Decimal::ZERO;
        };

        if self.scale <= real_scale {
            return *self;
        }

        let e = self.scale - real_scale;
        debug_assert!(e > 0);
        if e > MAX_PRECISION as i16 {
            return Decimal::ZERO;
        }

        let int_val = (self.int_val + ROUNDINGS[e as usize].low()) / POWERS_10[e as usize].low();

        unsafe { Decimal::from_parts_unchecked(int_val, real_scale, self.negative) }
    }

    /// Do bounds checking and rounding according to `precision` and `scale`.
    ///
    /// Returns `true` if overflows.
    #[inline]
    pub fn round_with_precision(&mut self, precision: u8, scale: i16) -> bool {
        if self.is_zero() {
            return false;
        }

        // N * 10^E < 10^(P - S)
        // => log(N) + E < P - S
        // => N < 10^(P - E - S)   N > 1
        // => P > E + S

        // E < P - S, E < 0
        let e = scale - self.scale;
        if e >= precision as i16 {
            return true;
        }

        if e < -(self.precision() as i16) {
            *self = Decimal::ZERO;
            return false;
        }

        // N * 10^E = N * 10^(E + S) * 10^ (-S)
        if e >= 0 {
            let ceil = POWERS_10[(precision as i16 - e) as usize].low();
            if self.int_val >= ceil {
                return true;
            }

            if e == 0 {
                return false;
            }

            let val = U256::mul128(self.int_val, POWERS_10[e as usize].low());
            self.int_val = val.low();
        } else {
            let div_result = U256::from(self.int_val).div128_round(POWERS_10[-e as usize].low());
            let ceil = POWERS_10[precision as usize].low();
            self.int_val = div_result.low();
            if self.int_val >= ceil {
                return true;
            }
        }

        self.scale = scale;
        false
    }

    /// Normalize a `Decimal`'s scale toward specified `scale`.
    #[inline]
    pub fn normalize_to_scale(&self, scale: i16) -> Decimal {
        if self.is_zero() {
            return Decimal::ZERO;
        }

        if self.scale == scale {
            return *self;
        }

        let mut current_scale = self.scale;
        let mut int_val = self.int_val;

        while current_scale > scale {
            if int_val % 10 > 0 {
                break;
            }

            int_val /= 10;
            current_scale -= 1;
        }

        while current_scale < scale {
            if int_val >= 10_0000_0000_0000_0000_0000_0000_0000_0000_0000_u128 {
                break;
            }

            int_val *= 10;
            current_scale += 1;
        }

        unsafe { Decimal::from_parts_unchecked(int_val, current_scale, self.negative) }
    }

    /// Normalize a `Decimal`'s scale toward zero.
    #[inline]
    pub fn normalize(&self) -> Decimal {
        self.normalize_to_scale(0)
    }

    #[inline]
    fn rescale_cmp(&self, other: &Decimal) -> Ordering {
        debug_assert!(self.scale < other.scale);

        let e = other.scale - self.scale;
        debug_assert!(e > 0);
        if e as u32 > MAX_PRECISION {
            Ordering::Greater
        } else {
            let self_int_val = U256::mul128(self.int_val, POWERS_10[e as usize].low());
            self_int_val.cmp128(other.int_val)
        }
    }

    #[inline]
    fn adjust_scale(int_val: U256, scale: i16, negative: bool) -> Option<Decimal> {
        let digits = int_val.count_digits();
        let s = scale as i32 - digits as i32;

        if s >= MAX_SCALE as i32 {
            return Some(Decimal::ZERO);
        }

        if s < MIN_SCALE as i32 {
            // overflow
            return None;
        }

        if digits > MAX_PRECISION {
            let shift_scale = (digits - MAX_PRECISION) as i16;
            return if shift_scale as u32 <= MAX_PRECISION {
                let dividend = int_val + ROUNDINGS[shift_scale as usize].low();
                let result = dividend / POWERS_10[shift_scale as usize].low();
                Some(unsafe { Decimal::from_parts_unchecked(result.low(), scale - shift_scale, negative) })
            } else {
                let dividend = int_val + ROUNDINGS[shift_scale as usize];
                let result = dividend / POWERS_10[shift_scale as usize];
                Some(unsafe { Decimal::from_parts_unchecked(result.low(), scale - shift_scale, negative) })
            };
        }

        Some(unsafe { Decimal::from_parts_unchecked(int_val.low(), scale, negative) })
    }

    #[inline]
    fn rescale_add(&self, other: &Decimal, negative: bool) -> Option<Decimal> {
        debug_assert!(self.scale < other.scale);

        let e = other.scale - self.scale;
        debug_assert!(e > 0);
        if e as u32 > MAX_PRECISION {
            if self.is_zero() {
                return Some(unsafe { Decimal::from_parts_unchecked(other.int_val, other.scale, negative) });
            }
            if other.is_zero() {
                return Some(unsafe { Decimal::from_parts_unchecked(self.int_val, self.scale, negative) });
            }
            if (e as usize) < POWERS_10.len() {
                if let Some(self_int_val) = POWERS_10[e as usize].checked_mul(self.int_val) {
                    if let Some(int_val) = self_int_val.checked_add(other.int_val) {
                        return Decimal::adjust_scale(int_val, other.scale, negative);
                    }
                }
            }

            return Some(unsafe { Decimal::from_parts_unchecked(self.int_val, self.scale, negative) });
        }

        let self_int_val = U256::mul128(self.int_val, POWERS_10[e as usize].low());
        let int_val = self_int_val + other.int_val;
        Decimal::adjust_scale(int_val, other.scale, negative)
    }

    #[inline]
    fn add_internal(&self, other: &Decimal, negative: bool) -> Option<Decimal> {
        if self.scale != other.scale {
            return if self.scale < other.scale {
                self.rescale_add(other, negative)
            } else {
                other.rescale_add(self, negative)
            };
        }

        let int_val = U256::add128(self.int_val, other.int_val);
        if !int_val.is_decimal_overflowed() && self.scale >= 0 {
            return Some(unsafe { Decimal::from_parts_unchecked(int_val.low(), self.scale, negative) });
        }

        Decimal::adjust_scale(int_val, self.scale, negative)
    }

    #[inline]
    fn rescale_sub(&self, other: &Decimal, negative: bool) -> Option<Decimal> {
        debug_assert!(self.scale < other.scale);

        let e = other.scale - self.scale;
        debug_assert!(e > 0);
        if e as u32 > MAX_PRECISION {
            if (e as usize) < POWERS_10.len() {
                if let Some(self_int_val) = POWERS_10[e as usize].checked_mul(self.int_val) {
                    if let Some(int_val) = self_int_val.checked_sub(other.int_val) {
                        return Decimal::adjust_scale(int_val, other.scale, negative);
                    }
                }
            }

            return Some(unsafe { Decimal::from_parts_unchecked(self.int_val(), self.scale, negative) });
        }

        let self_int_val = U256::mul128(self.int_val(), POWERS_10[e as usize].low());
        let (int_val, neg) = if self_int_val >= other.int_val() {
            let result = self_int_val - other.int_val();
            (result, negative)
        } else {
            let result = other.int_val() - self_int_val;
            (U256::from(result), !negative)
        };

        Decimal::adjust_scale(int_val, other.scale, neg)
    }

    #[inline]
    fn sub_internal(&self, other: &Decimal, negative: bool) -> Option<Decimal> {
        if other.int_val == 0 {
            return Some(*self);
        }

        if self.int_val == 0 {
            return Some(unsafe { Decimal::from_parts_unchecked(other.int_val, other.scale, !negative) });
        }

        if self.scale != other.scale {
            return if self.scale < other.scale {
                self.rescale_sub(other, negative)
            } else {
                other.rescale_sub(self, !negative)
            };
        }

        debug_assert_eq!(self.scale, other.scale);
        let (val, neg) = if self.int_val >= other.int_val {
            (self.int_val - other.int_val, negative)
        } else {
            (other.int_val - self.int_val, !negative)
        };

        Some(unsafe { Decimal::from_parts_unchecked(val, self.scale, neg) })
    }

    /// Add two decimals,
    /// returning `None` if overflow occurred.
    #[inline]
    pub fn checked_add(&self, other: impl AsRef<Decimal>) -> Option<Decimal> {
        let other = other.as_ref();
        if self.negative != other.negative {
            if other.negative {
                self.sub_internal(other, self.negative)
            } else {
                other.sub_internal(self, other.negative)
            }
        } else {
            self.add_internal(other, self.negative)
        }
    }

    /// Subtract one decimal from another,
    /// returning `None` if overflow occurred.
    #[inline]
    pub fn checked_sub(&self, other: impl AsRef<Decimal>) -> Option<Decimal> {
        let other = other.as_ref();
        if self.negative != other.negative {
            self.add_internal(other, self.negative)
        } else if self.negative {
            other.sub_internal(self, !self.negative)
        } else {
            self.sub_internal(other, self.negative)
        }
    }

    /// Calculate the product of two decimals,
    /// returning `None` if overflow occurred.
    #[inline]
    pub fn checked_mul(&self, other: impl AsRef<Decimal>) -> Option<Decimal> {
        let other = other.as_ref();

        if self.is_zero() || other.is_zero() {
            return Some(Decimal::ZERO);
        }

        let scale = self.scale + other.scale;
        let negative = self.negative ^ other.negative;
        let int_val = U256::mul128(self.int_val, other.int_val);

        if !int_val.is_decimal_overflowed() && scale == 0 {
            Some(unsafe { Decimal::from_parts_unchecked(int_val.low(), 0, negative) })
        } else {
            Decimal::adjust_scale(int_val, scale, negative)
        }
    }

    /// Checked decimal division.
    /// Computes `self / other`, returning `None` if `other == 0` or the division results in overflow.
    #[inline]
    pub fn checked_div(&self, other: impl AsRef<Decimal>) -> Option<Decimal> {
        let other = other.as_ref();

        if other.is_zero() {
            return None;
        }

        if self.is_zero() {
            return Some(Decimal::ZERO);
        }

        let other_precision = other.precision();
        let self_precision = self.precision();

        let (self_int_val, shift_precision) = if other_precision > self_precision {
            let p = MAX_PRECISION + (other_precision - self_precision) as u32;
            (POWERS_10[p as usize] * self.int_val, other_precision - self_precision)
        } else {
            (U256::mul128(self.int_val, POWERS_10[MAX_PRECISION as usize].low()), 0)
        };

        let negative = self.negative ^ other.negative;
        let int_val = self_int_val.div128_round(other.int_val);
        let scale = self.scale - other.scale + MAX_PRECISION as i16 + shift_precision as i16;

        Decimal::adjust_scale(int_val, scale, negative)
    }

    /// Checked decimal remainder.
    /// Computes `self % other`, returning None if rhs == 0 or the division results in overflow.
    #[inline]
    pub fn checked_rem(&self, other: impl AsRef<Decimal>) -> Option<Decimal> {
        let other = other.as_ref();

        if other.is_zero() {
            return None;
        }

        if self.is_zero() {
            return Some(Decimal::ZERO);
        }

        if self.scale == other.scale {
            let rem = self.int_val % other.int_val;
            return Some(unsafe { Decimal::from_parts_unchecked(rem, self.scale, self.negative) });
        }

        if self.scale < other.scale {
            let e = other.scale - self.scale;
            debug_assert!(e > 0);

            let mut res = *self;
            loop {
                let scale = (MAX_PRECISION as i16).min(other.scale - res.scale);
                let res_val = U256::mul128(res.int_val, POWERS_10[scale as usize].low());
                let rem = res_val % other.int_val;
                res = unsafe { Decimal::from_parts_unchecked(rem.low(), res.scale + scale, res.negative) };
                if res.scale == other.scale || res.is_zero() {
                    break;
                }
            }
            Some(res)
        } else {
            let e = self.scale - other.scale;
            debug_assert!(e > 0);
            if e as u32 > MAX_PRECISION {
                return Some(*self);
            }

            let other_int_val = U256::mul128(other.int_val, POWERS_10[e as usize].low());
            let rem = self.int_val % other_int_val;
            debug_assert_eq!(rem.high(), 0);

            Some(unsafe { Decimal::from_parts_unchecked(rem.low(), self.scale, self.negative) })
        }
    }

    /// Computes the square root of a decimal,
    /// returning None if `self` is negative or the results in overflow.
    #[inline]
    pub fn sqrt(&self) -> Option<Decimal> {
        if self.negative {
            return None;
        }

        if self.is_zero() {
            return Some(Decimal::ZERO);
        }

        let mut result = Decimal::ONE;
        let mut last = result;

        loop {
            let val = self.checked_div(&result)?.normalize();
            result = result.checked_add(&val)?;
            result = result.checked_mul(&Decimal::ZERO_POINT_FIVE)?;

            if result == last {
                break;
            }

            last = result;
        }

        Some(result)
    }

    /// Formats the decimal, including sign and omitting integer zero in fractional.
    #[inline]
    pub fn simply_format<W: fmt::Write>(&self, w: W) -> Result<(), DecimalFormatError> {
        self.fmt_internal(true, true, true, None, w)
    }

    #[inline]
    pub(crate) fn fmt_internal<W: fmt::Write>(
        &self,
        append_sign: bool,
        omit_integer_zero: bool,
        omit_frac_ending_zero: bool,
        precision: Option<usize>,
        mut w: W,
    ) -> Result<(), DecimalFormatError> {
        use std::fmt::Write;

        const ZERO_BUF: [u8; 256] = [b'0'; 256];

        if self.is_zero() {
            w.write_byte(b'0')?;
            return Ok(());
        }

        let dec = if let Some(prec) = precision {
            self.round(prec as i16)
        } else {
            *self
        };

        let scale = dec.scale();

        if append_sign && self.is_sign_negative() {
            w.write_byte(b'-')?;
        }

        if scale <= 0 {
            write!(w, "{}", dec.int_val())?;
            w.write_bytes(&ZERO_BUF[..-scale as usize])?;
            if let Some(prec) = precision {
                if prec != 0 {
                    w.write_byte(b'.')?;
                    w.write_bytes(&ZERO_BUF[..prec])?;
                }
            }
        } else {
            let mut buf = StackVec::<u8, 40>::new();
            write!(&mut buf, "{}", dec.int_val())?;
            let digits = buf.as_slice();

            let len = digits.len();
            if len <= scale as usize {
                if !omit_integer_zero {
                    w.write_byte(b'0')?;
                }
                w.write_byte(b'.')?;
                w.write_bytes(&ZERO_BUF[..scale as usize - len])?;
                if omit_frac_ending_zero {
                    let zero_num = digits.iter().rev().take_while(|ch| **ch == b'0').count();
                    w.write_bytes(&digits[0..len - zero_num])?;
                } else {
                    w.write_bytes(digits)?;
                }
            } else {
                let (int_digits, frac_digits) = digits.split_at(len - scale as usize);
                w.write_bytes(int_digits)?;
                if let Some(prec) = precision {
                    w.write_byte(b'.')?;
                    let after_len = frac_digits.len();
                    if prec > after_len {
                        w.write_bytes(frac_digits)?;
                        w.write_bytes(&ZERO_BUF[..prec - after_len])?;
                    } else {
                        w.write_bytes(&frac_digits[0..prec])?;
                    }
                } else {
                    let zero_num = frac_digits.iter().rev().take_while(|ch| **ch == b'0').count();
                    if zero_num < frac_digits.len() {
                        w.write_byte(b'.')?;
                        w.write_bytes(&frac_digits[0..frac_digits.len() - zero_num])?;
                    }
                }
            }
        }

        Ok(())
    }

    #[inline]
    fn fmt_sci_internal<W: fmt::Write, const POSITIVE_EXP: bool, const MIN_SCALE: i16>(
        &self,
        expect_scale: i16,
        mut exp: u16,
        mut w: W,
    ) -> Result<(), DecimalFormatError> {
        if expect_scale >= MIN_SCALE {
            // Creates number part
            let temp_scale = if POSITIVE_EXP {
                expect_scale - exp as i16
            } else {
                expect_scale + exp as i16
            };

            let mut dec = self.round(temp_scale);

            // Whether number carries or not
            if dec.precision() > self.trunc(temp_scale).precision() {
                if POSITIVE_EXP {
                    exp += 1
                } else {
                    exp -= 1
                }
            }

            // This decimal only includes scientific notation number part
            if POSITIVE_EXP {
                dec.scale += exp as i16
            } else {
                dec.scale -= exp as i16
            };

            // Supplies zero to fill expect scale
            dec.fmt_internal(true, true, true, Some(expect_scale as usize), &mut w)?;

            if POSITIVE_EXP {
                write_exp(b"E+", exp, true, w)?;
            } else {
                write_exp(b"E-", exp, true, w)?;
            }
        } else {
            return Err(DecimalFormatError::OutOfRange);
        }

        Ok(())
    }

    /// Formats the decimal, using scientific notation depending on the width.
    #[inline]
    pub fn format_with_sci<W: fmt::Write>(&self, max_width: u16, mut w: W) -> Result<(), DecimalFormatError> {
        const DOT_LEN: u16 = 1; // the length of "."

        if self.is_zero() {
            w.write_byte(b'0')?;
            return Ok(());
        }

        let precision = self.precision() as i16;
        let sign_len = if self.negative { 1 } else { 0 };
        // include ".", but without sign
        let max_digits = max_width - sign_len;

        let (use_sci, positive_exp, prec): (bool, bool, Option<usize>) = if self.scale < precision {
            // integer part
            let int_len = (precision - self.scale) as u16;
            if max_digits >= int_len {
                if max_digits == int_len {
                    (false, true, Some(0))
                } else {
                    // length of the fractional part
                    let scale = (max_digits as u16 - int_len - DOT_LEN) as usize;
                    if scale as i16 >= self.scale() {
                        (false, true, None)
                    } else {
                        (false, true, Some(scale))
                    }
                }
            } else {
                // use sci notation, with "E+"
                (true, true, None)
            }
        } else if self.scale - precision >= 5 {
            if max_digits < self.scale as u16 + DOT_LEN {
                // use sci notation, with "E-"
                (true, false, None)
            } else {
                (false, true, None)
            }
        } else {
            // round the decimal
            let scale = max_width as usize - 1;
            (false, true, Some(scale))
        };

        if use_sci {
            const E_NOTATION_LEN: usize = 2; // "E+" or "E-"
            const SCI_INT_LEN: i16 = 2; // e.g. "1."

            // Ignore the sign in exponent part
            let exp = (precision - self.scale - 1).abs() as u16;
            // 'E' + sign + exponent number
            let exp_len = E_NOTATION_LEN + if exp < 100 { 2 } else { 3 };
            // Remove integer and '.' in scientific notation
            let expect_scale = max_digits as i16 - exp_len as i16 - SCI_INT_LEN;

            const MIN_SCALE: i16 = 1;
            if positive_exp {
                self.fmt_sci_internal::<W, true, MIN_SCALE>(expect_scale, exp, w)?;
            } else {
                self.fmt_sci_internal::<W, false, MIN_SCALE>(expect_scale, exp, w)?;
            }
        } else {
            self.fmt_internal(true, true, true, prec, w)?;
        }

        Ok(())
    }

    /// Formats the decimal, forced using scientific notation depending on the scale.
    ///
    /// In particular, the scientific notation is also enforced for 0.  
    /// When the decimal is 0 and expect_scale greater than 0, with_zero_before_dot determines whether there is a 0 before the decimal point.
    #[inline]
    pub fn format_with_sci_forced<W: fmt::Write>(
        &self,
        expect_scale: i16,
        with_zero_before_dot: bool,
        mut w: W,
    ) -> Result<(), DecimalFormatError> {
        // max_scale: 64(to_char max length) - 1(sign) - 1(.) -1(integer_count) - 5 = 56
        const MAX_SCALE: usize = 56;
        if expect_scale > MAX_SCALE as i16 {
            return Err(DecimalFormatError::OutOfRange);
        }
        let precision = self.precision() as i16;
        let exp = (precision - self.scale - 1).abs() as u16;
        let positive_exp = precision > self.scale;

        if self.is_zero() && expect_scale > 0 {
            const ZERO_BUF: [u8; MAX_SCALE] = [b'0'; MAX_SCALE];
            if with_zero_before_dot {
                w.write_bytes(b"0.")?;
            } else {
                w.write_bytes(b" .")?;
            }
            w.write_bytes(&ZERO_BUF[..expect_scale as usize - 1])?;
        }

        const MIN_SCALE: i16 = 0;
        if positive_exp {
            self.fmt_sci_internal::<W, true, MIN_SCALE>(expect_scale, exp, w)?;
        } else {
            self.fmt_sci_internal::<W, false, MIN_SCALE>(expect_scale, exp, w)?;
        }
        Ok(())
    }

    /// Format decimal as a hexadecimal number.
    ///
    /// A maximum of 63 digits hexadecimal positive number are supported.
    #[inline]
    pub fn format_to_hex<W: fmt::Write>(&self, is_uppercase: bool, mut w: W) -> Result<(), DecimalFormatError> {
        // Max number: u256::MAX/16 = 7237005577332262213973186563042994240829374041602535252466099000494570602495
        const MAX_DECIMAL: Decimal =
            unsafe { Decimal::from_parts_unchecked(72370055773322622139731865630429942408, -38, false) };

        if self.is_sign_negative() || self > MAX_DECIMAL {
            return Err(DecimalFormatError::OutOfRange);
        }

        let integer = self.round(0);
        let real_num = POWERS_10[(-integer.scale) as usize] * integer.int_val;
        if is_uppercase {
            if real_num.high() != 0 {
                write!(&mut w, "{:X}", real_num.high())?;
            }
            write!(&mut w, "{:X}", real_num.low())?;
        } else {
            if real_num.high() != 0 {
                write!(&mut w, "{:x}", real_num.high())?;
            }
            write!(&mut w, "{:x}", real_num.low())?;
        }

        Ok(())
    }

    /// Formats the decimal in the json number format, using scientific notation depending on the width.
    #[inline]
    pub fn format_to_json<W: fmt::Write>(&self, mut w: W) -> Result<(), DecimalFormatError> {
        if self.is_zero() {
            w.write_byte(b'0')?;
            return Ok(());
        }

        const MAX_WIDTH: i16 = 40;

        let precision = self.precision() as i16;
        let use_sci = if self.scale <= 0 {
            precision - self.scale > MAX_WIDTH
        } else {
            let mut int_val = self.int_val;
            let mut zero_count = 0;
            while int_val != 0 {
                if int_val % 10 != 0 {
                    break;
                }
                zero_count += 1;
                int_val /= 10;
            }
            self.scale - zero_count > MAX_WIDTH
        };

        if !use_sci {
            return self.fmt_internal(true, false, true, None, w);
        }

        let mut dec = *self;
        let positive_exp = precision > dec.scale;
        let exp = (precision - dec.scale - 1).abs() as u16;
        if positive_exp {
            dec.scale += exp as i16;
            dec.fmt_internal(true, false, true, None, &mut w)?;
            write_exp(b"E+", exp, false, w)?;
        } else {
            dec.scale -= exp as i16;
            dec.fmt_internal(true, false, true, None, &mut w)?;
            write_exp(b"E-", exp, false, w)?;
        };

        Ok(())
    }

    /// Raise `self` to the power of `exponent`, where `self`
    /// is a decimal and `exponent` is an u64 integer,
    /// returning None if the result overflowed.
    #[inline]
    fn pow_u64(&self, exponent: u64) -> Option<Decimal> {
        match exponent {
            0 => Some(Decimal::ONE),
            1 => Some(*self),
            2 => self.checked_mul(self),
            _ => {
                // Here use Exponentiation by squaring to calculate x^n:
                // Let a + b + c + ... = n,
                //   x^n = x^(a + b + c + ...) = x^a * x^b * x^c * ...
                // Here a, b, c ... are powers of 2,
                // so x^a, x^b, x^c ... can be calculated by squaring x.

                let x = *self;
                let mut n = exponent;
                let mut sum = Decimal::ONE;
                let mut power_x = x;

                // Multiply once to avoid power_x greater than x^n,
                // so power_x will not cross the boundary first.
                if n & 1 == 1 {
                    sum = sum.checked_mul(&power_x)?;
                }
                n >>= 1;

                while n != 0 {
                    power_x = power_x.checked_mul(&power_x)?;
                    if n & 1 == 1 {
                        sum = sum.checked_mul(&power_x)?;
                    }
                    n >>= 1;
                }

                Some(sum)
            }
        }
    }

    /// The range that Decimal can represent `self` to the power of |`exponent`|,
    /// where `exponent` is negative, only used in `pow_i64()` to calculate quickly.
    #[inline]
    fn pow_quick_range(&self, exponent: u64) -> bool {
        // 1163^42 won't overflow, 1164^42 and 1163^43 will overflow, so 1163^42 is an upper
        // bound, `self` to the power of -`exponent` in this range can be calculated quickly.
        // 125^61 won't overflow, 126^61 and 125^62 will overflow, so 125^61 is an upper
        // bound, `self` to the power of -`exponent` in this range can be calculated quickly.
        // 10^126 won't overflow, 11^126 and 10^127 will overflow, so 10^126 is an upper
        // bound, `self` to the power of -`exponent` in this range can be calculated quickly.

        const BASE_UPPER_BOUND1: Decimal = unsafe { Decimal::from_parts_unchecked(1163, 0, false) };
        const EXP_UPPER_BOUND1: u64 = 42;
        const BASE_UPPER_BOUND2: Decimal = unsafe { Decimal::from_parts_unchecked(125, 0, false) };
        const EXP_UPPER_BOUND2: u64 = 61;
        const BASE_UPPER_BOUND3: Decimal = unsafe { Decimal::from_parts_unchecked(1, -1, false) };
        const EXP_UPPER_BOUND3: u64 = 126;

        (exponent < EXP_UPPER_BOUND1 && *self < BASE_UPPER_BOUND1)
            || (exponent < EXP_UPPER_BOUND2 && *self < BASE_UPPER_BOUND2)
            || (exponent < EXP_UPPER_BOUND3 && *self < BASE_UPPER_BOUND3)
    }

    /// Raise `self` to the power of `exponent`, where `self` is
    /// a decimal and `exponent` is an i64 integer, returning None
    /// if `self == 0` at the same time `exponent` is negative or
    /// the result overflowed.
    #[inline]
    fn pow_i64(&self, exponent: i64) -> Option<Decimal> {
        if exponent >= 0 {
            return self.pow_u64(exponent as u64);
        }
        // exponent is negative, example: 0^-3 is error
        if self.is_zero() {
            return None;
        }

        // Here use reciprocal value to calculate x^-y:
        //   x^-y = 1 / x^y
        // Here y is positive, so can calculate x^y from `pow_u64()`.

        let x = *self;
        let y = exponent.unsigned_abs();

        // x and y in some ranges can be calculated quickly.
        let result = if x.pow_quick_range(y) {
            // x^y won't overflow, so can be calculated quickly
            Decimal::ONE.checked_div(&x.pow_u64(y)?)?
        } else {
            // x^y maybe overflow, so calculate x^-y with x^(y/2)

            // if y is even,
            //   x^-y = 1 / x^y = 1 / x^(y/2) / x^(y/2)
            // if y is odd,
            //   x^-y = 1 / x^y = 1 / x^(y/2) / x^(y/2) / x

            match x.pow_u64(y / 2) {
                Some(p) => {
                    let power = Decimal::ONE.checked_div(&p)?.checked_div(p)?;
                    if y % 2 == 1 {
                        power.checked_div(&x)?
                    } else {
                        power
                    }
                }
                // x^(y/2) is overflow, x^-y = 1 / x^(y/2) / x^(y/2) must be 0
                None => Decimal::ZERO,
            }
        };

        Some(result)
    }

    /// Raise `self` to the power of `exponent`, where `self`
    /// and `exponent` are both decimal, requires `exponent`
    /// is an integer, only used in `checked_pow()`.
    #[inline]
    fn pow_decimal_integral(&self, exponent: &Decimal) -> Option<Decimal> {
        debug_assert!((exponent.int_val == exponent.normalize().int_val) && (exponent.scale() <= 0));

        if exponent.is_sign_negative() {
            // too small to calculate from pow_i64 accurately
            if *exponent < Decimal::from(i16::MIN) {
                return self.pow_decimal(exponent);
            }

            self.pow_i64(-(exponent.int_val as i64))
        } else {
            // too big to calculate from pow_u64 accurately
            if *exponent > Decimal::from(u16::MAX) {
                return self.pow_decimal(exponent);
            }

            self.pow_u64(exponent.int_val as u64)
        }
    }

    /// Raise `self` to the power of `exponent`, where `self` and
    /// `exponent` are both decimal, only used in `checked_pow()`,
    /// requires `self` is positive or `exponent` is an integer,
    /// returning None if the result overflowed.
    #[inline]
    fn pow_decimal(&self, exponent: &Decimal) -> Option<Decimal> {
        debug_assert!((*self > Decimal::ZERO) || (exponent.normalize().scale() <= 0));

        // For positive x:
        //   x^b = e^(b * ln(x))
        // If x is negative, calculate |x|^b then add a sign.
        // When x is negative and b is odd, x^b will be negative.
        // When x is negative and b is even, x^b will be positive.

        let x = self.abs();
        let b = *exponent;

        let ln = x.ln()?;
        let exp = ln.checked_mul(&b)?;
        let mut result = exp.exp()?;

        if !self.negative && b.checked_rem(&Decimal::TWO)? == Decimal::ONE {
            result = -result;
        }

        Some(result)
    }

    /// Raise `self` to the power of `exponent`, where `self` and `exponent`
    /// are both decimal, returning None if `self == 0` at the same time
    /// `exponent` is negative or `self` is negative at the same time
    /// `exponent` is a fraction or the result overflowed.
    #[inline]
    pub fn checked_pow(&self, exponent: &Decimal) -> Option<Decimal> {
        if exponent.is_zero() {
            return Some(Decimal::ONE);
        }
        if self.is_zero() {
            // exponent is negative, example: 0^-3 is error
            if exponent.is_sign_negative() {
                return None;
            }
            return Some(Decimal::ZERO);
        }
        if *self == Decimal::ONE {
            return Some(Decimal::ONE);
        }
        if exponent == Decimal::ONE {
            return Some(*self);
        }

        let exponent = exponent.normalize();
        // exponent is an integer
        if exponent.scale() <= 0 {
            return self.pow_decimal_integral(&exponent);
        }

        // base is negative and exponent is a fraction, example: (-3)^2.2 is error
        if self.is_sign_negative() {
            return None;
        }

        // Let n = a + b:
        //   x^n = x^(a + b) = x^a * x^b,
        // where a is the integer part of n and b is the fraction part of n.
        // a is an integer and b is a fraction in range (-1, 1),
        // so calculate x^a and x^b is faster and more accurate.

        let x = *self;
        let n = exponent;

        let a = n.trunc(0);
        let b = n.checked_sub(&a)?;

        let power_a = x.pow_decimal_integral(&a)?;
        let power_b = x.pow_decimal(&b)?;

        // x^n = x^(a + b) = x^a * x^b
        let result = power_a.checked_mul(&power_b)?;

        Some(result)
    }

    /// Computes the natural logarithm of `self`,
    /// returning None if `self` is negative or `self == 0`.
    #[inline]
    pub fn ln(&self) -> Option<Decimal> {
        const ZERO_POINT_ONE: Decimal = unsafe { Decimal::from_parts_unchecked(1, 1, false) };
        const ONE_POINT_ONE: Decimal = unsafe { Decimal::from_parts_unchecked(11, 1, false) };
        const TEN: Decimal = unsafe { Decimal::from_parts_unchecked(10, 0, false) };
        const LOWER_BOUND: Decimal = unsafe { Decimal::from_parts_unchecked(9047, 4, false) };
        // 1.2217
        const R: Decimal = unsafe { Decimal::from_parts_unchecked(12217, 4, false) };
        const LN_10: Decimal =
            unsafe { Decimal::from_parts_unchecked(23025850929940456840179914546843642076, 37, false) };
        // ln(1.2217)
        const LN_R: Decimal =
            unsafe { Decimal::from_parts_unchecked(2002433314278771112016301166984297937, 37, false) };

        // ln(x) requires x > 0
        if self.is_sign_negative() || self.is_zero() {
            return None;
        }

        if *self == Decimal::ONE {
            return Some(Decimal::ZERO);
        }

        // Taylor series:
        //   ln(x) = ln((1 + y) / (1 - y)) = 2(y + y^3/3 + y^5/5 + y^7 / 7 + ...)
        // The Taylor series converges fast as y approaches 0.
        //
        // ln(x) = ln(x / 10^n1 * 10^n1) = ln(x / 10^n1) + n1 * ln(10),
        // ln(x / 10^n1) = ln(x / 10^n1 / R^n2 * R^n2) = ln(x / 10^n1 / R^n2) + n2 * ln(R),
        // let z = x / 10^n1 / R^n2, then ln(x) = ln(z) + n1 * ln(10) + n2 * ln(R)
        //
        // Here use Taylor series to calculate ln(z).
        // let z = (1 + y)/(1 - y), for requires y in (-0.05, 0.05)(this range approaches 0),
        // lower bound of z is (1 + -0.05) / (1 - -0.05) = 0.9047,
        // upper bound of z is (1 + 0.05) / (1 - 0.05) = 1.10526,
        // so need reduce x into z in range [0.9047, 1.10526),
        // R = 1.10526 / 0.9047 = 1.2217.

        let mut x = *self;
        let mut n1 = 0;
        let mut n2 = 0;

        // reduce x into (0.1, 1.1]
        while x > ONE_POINT_ONE {
            x = x.checked_mul(&ZERO_POINT_ONE)?;
            n1 += 1;
        }
        while x <= ZERO_POINT_ONE {
            x = x.checked_mul(&TEN)?;
            n1 -= 1;
        }

        // reduce x into [0.9047, 1.10526)
        while x < LOWER_BOUND {
            x = x.checked_mul(&R)?;
            n2 -= 1;
        }

        // z = (1 + y)/(1 - y), then y = (z - 1)/(z + 1)
        let z = x;
        let y = z
            .checked_sub(&Decimal::ONE)?
            .checked_div(&z.checked_add(&Decimal::ONE)?)?;
        let y_square = y.checked_mul(&y)?;

        // ln(z) = ln((1 + y)/(1 - y)) = 2 * (y + y^3 / 3 + y^5 / 5 + y^7 / 7 + ...)
        let mut sum = y;
        let mut power_y = y;
        let mut last;
        let mut iter = 1;

        loop {
            iter += 2;
            power_y = power_y.checked_mul(&y_square)?;
            let term = power_y.checked_div(&Decimal::from(iter))?;

            if term.is_zero() {
                break;
            }

            last = sum;
            sum = sum.checked_add(&term)?;

            if last == sum {
                break;
            }
        }

        let ln_z = sum.checked_mul(&Decimal::TWO)?;

        // ln(x) = ln(z) + n1 * ln(10) + n2 * ln(R).
        let mut result = ln_z.checked_add(&LN_10.checked_mul(&Decimal::from(n1))?)?;
        result = result.checked_add(&LN_R.checked_mul(&Decimal::from(n2))?)?;
        Some(result)
    }

    /// Computes the nature exponential of `self`,
    /// calculate with Taylor series, returning
    /// None if the result overflowed.
    fn exp_decimal(&self) -> Option<Decimal> {
        // Taylor series:
        //   e^x = 1 + x + x^2 / 2! + x^3 / 3! + x^4 / 4! + ...
        // Here use Taylor series to calculate e^x,
        // start with the third term.

        let x = *self;
        let mut term = x;
        let mut sum = Decimal::ONE.checked_add(&x)?;
        let mut last;
        let mut iter = 1;
        loop {
            iter += 1;

            // Calculate latter term from former term by multiplying x over iter,
            // Divide first then multiply to avoid the intermediate process to cross the boundary.
            term = term.checked_div(&Decimal::from(iter))?.checked_mul(&x)?;

            if term.is_zero() {
                break;
            }

            last = sum;
            sum = sum.checked_add(&term)?;

            if last == sum {
                break;
            }
        }

        Some(sum)
    }

    /// Computes the nature exponential of `self`,
    /// returning None if the result overflowed.
    #[inline]
    pub fn exp(&self) -> Option<Decimal> {
        // same as Oracle: e^291 will overflow, e^-300 is 0
        const UPPER_BOUND: Decimal = unsafe { Decimal::from_parts_unchecked(291, 0, false) };
        const LOWER_BOUND: Decimal = unsafe { Decimal::from_parts_unchecked(300, 0, true) };

        if self.is_zero() {
            return Some(Decimal::ONE);
        }
        if *self >= UPPER_BOUND {
            // overflow
            return None;
        }
        if *self <= LOWER_BOUND {
            return Some(Decimal::ZERO);
        }

        // Taylor series:
        //   e^x = 1 + x + x^2 / 2! + x^3 / 3! + x^4 / 4! + ...
        // The Taylor series converges faster as input approaches 0,
        //
        // Let x = a + b:
        //   e^x = e^(a + b) = e^a * e^b,
        // where a is the integer part of x and b is the fraction part of x,
        // to reduce input into range -1 < b < 1 by getting rid of the integer part of x.
        //
        // Here use look-up table to get e^a,
        // calculate e^a in advance when testing by using Taylor series,
        // put it into array `NATURAL_EXP` and `NATURAL_EXP_NEG`.
        //
        // Here use Taylor series to calculate e^b,
        // b is the fraction part of x, so b is in (-1, 1)(this range approaches 0).

        let x = *self;
        let a = x.trunc(0);
        let b = x.checked_sub(&a)?;

        let exp_a = if a.is_sign_positive() {
            NATURAL_EXP[a.int_val as usize]
        } else if a.int_val < UPPER_BOUND.int_val {
            // e^|a| won't overflow
            Decimal::ONE.checked_div(&NATURAL_EXP[a.int_val as usize])?
        } else {
            // e^|a| will overflow
            NATURAL_EXP_NEG[(a.int_val - UPPER_BOUND.int_val) as usize]
        };

        let exp_b = if b.is_zero() {
            // e^0 = 1, so e^x = e^a.
            return Some(exp_a);
        } else {
            b.exp_decimal()?
        };

        // e^x = e^(a + b) = e^a * e^b
        let result = exp_a.checked_mul(&exp_b)?;

        Some(result)
    }
}

trait WriteExt: fmt::Write {
    #[inline(always)]
    fn write_byte(&mut self, byte: u8) -> fmt::Result {
        self.write_bytes(&[byte])
    }

    #[inline(always)]
    fn write_bytes(&mut self, bytes: &[u8]) -> fmt::Result {
        let s = unsafe { std::str::from_utf8_unchecked(bytes) };
        self.write_str(s)
    }
}

impl<W: fmt::Write> WriteExt for W {}

#[inline]
fn write_exp<W: fmt::Write>(
    e_notation: &[u8],
    exp: u16,
    add_left_padding_zero: bool,
    mut w: W,
) -> Result<(), DecimalFormatError> {
    w.write_bytes(e_notation)?;

    // Creates a temp array to save exp str
    let mut buf = [b'0'; 3];
    let mut index = 2;

    let mut val = exp;
    while val >= 10 {
        let v = val % 10;
        val /= 10;
        buf[index] += v as u8;
        index -= 1;
    }
    buf[index] += val as u8;

    // Adds zero if exponent number doesn't have two digits
    if index == 2 && add_left_padding_zero {
        index -= 1;
    }

    w.write_bytes(&buf[index..])?;
    Ok(())
}

impl AsRef<Decimal> for Decimal {
    #[inline]
    fn as_ref(&self) -> &Decimal {
        self
    }
}

impl fmt::Display for Decimal {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut buf = Buf::new();
        self.fmt_internal(false, false, false, f.precision(), &mut buf)
            .expect("failed to format decimal");
        let str = unsafe { std::str::from_utf8_unchecked(buf.as_slice()) };
        f.pad_integral(self.is_sign_positive(), "", str)
    }
}

impl Default for Decimal {
    #[inline]
    fn default() -> Self {
        Decimal::ZERO
    }
}

impl PartialEq for Decimal {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other) == Ordering::Equal
    }
}

impl PartialEq<&Decimal> for Decimal {
    #[inline]
    fn eq(&self, other: &&Decimal) -> bool {
        self.eq(*other)
    }
}

impl PartialEq<Decimal> for &Decimal {
    #[inline]
    fn eq(&self, other: &Decimal) -> bool {
        (*self).eq(other)
    }
}

impl PartialOrd for Decimal {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialOrd<&Decimal> for Decimal {
    #[inline]
    fn partial_cmp(&self, other: &&Decimal) -> Option<Ordering> {
        self.partial_cmp(*other)
    }
}

impl PartialOrd<Decimal> for &Decimal {
    #[inline]
    fn partial_cmp(&self, other: &Decimal) -> Option<Ordering> {
        (*self).partial_cmp(other)
    }
}

impl Ord for Decimal {
    #[inline]
    fn cmp(&self, other: &Self) -> Ordering {
        // sign is different
        if self.negative != other.negative {
            return if self.negative {
                Ordering::Less
            } else {
                Ordering::Greater
            };
        }

        let (left, right) = if self.negative {
            // both are negative, so reverse cmp
            debug_assert!(other.negative);
            (other, self)
        } else {
            (self, other)
        };

        if left.is_zero() {
            return if right.is_zero() {
                Ordering::Equal
            } else {
                Ordering::Less
            };
        } else if right.is_zero() {
            return Ordering::Greater;
        }

        if left.scale == right.scale {
            // fast path for same scale
            return left.int_val().cmp(&right.int_val());
        }

        if left.scale < right.scale {
            left.rescale_cmp(right)
        } else {
            right.rescale_cmp(left).reverse()
        }
    }
}

impl Hash for Decimal {
    #[inline]
    fn hash<H: Hasher>(&self, state: &mut H) {
        let n = self.normalize();
        n.int_val().hash(state);
        n.scale.hash(state);
        n.negative.hash(state);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decimal_repr() {
        assert_eq!(std::mem::size_of::<Decimal>(), 20);
        assert_eq!(std::mem::align_of::<Decimal>(), 4);
    }

    #[test]
    fn test_fmt_internal() {
        fn assert(
            int_val: u128,
            scale: i16,
            negative: bool,
            append_sign: bool,
            precision: Option<usize>,
            expected: &str,
        ) {
            let dec = Decimal::from_parts(int_val, scale, negative).unwrap();
            let mut buf = Buf::new();
            dec.fmt_internal(append_sign, false, false, precision, &mut buf)
                .unwrap();
            let str = unsafe { std::str::from_utf8_unchecked(buf.as_slice()) };
            assert_eq!(str, expected);
        }

        assert(128, 0, false, false, None, "128");
        assert(128, -2, true, true, None, "-12800");
        assert(128, 4, true, true, None, "-0.0128");
        assert(128, 2, true, false, None, "1.28");
        assert(1280, 4, true, true, None, "-0.1280");
        assert(12856, 4, true, false, None, "1.2856");
        assert(12856, 4, true, false, Some(2), "1.29");
        assert(12856, 4, true, false, Some(6), "1.285600");
        assert(1285600, 6, false, false, None, "1.2856");
    }

    #[test]
    fn test_display() {
        macro_rules! assert_display {
            ($num: expr, $scale: expr, $negative: expr, $fmt: expr,$expected: expr) => {{
                let dec = Decimal::from_parts($num, $scale, $negative).unwrap();
                let str = format!($fmt, dec);
                assert_eq!(str, $expected);
            }};
        }

        assert_display!(0, -1, false, "{}", "0");
        assert_display!(1, 0, false, "{}", "1");
        assert_display!(1, 1, false, "{}", "0.1");
        assert_display!(1, -1, false, "{}", "10");
        assert_display!(10, 0, false, "{}", "10");
        assert_display!(10, 1, false, "{}", "1");
        assert_display!(10, -1, false, "{}", "100");
        assert_display!(128, 0, false, "{}", "128");
        assert_display!(128, -2, true, "{}", "-12800");
        assert_display!(128, 4, true, "{}", "-0.0128");
        assert_display!(128, 2, true, "{}", "-1.28");
        assert_display!(12800, 1, false, "{}", "1280");
        assert_display!(12800, 2, false, "{}", "128");
        assert_display!(12800, 3, false, "{}", "12.8");
        assert_display!(12856, 4, true, "{}", "-1.2856");
        assert_display!(12856, 4, true, "{:.2}", "-1.29");
        assert_display!(12856, 4, true, "{:.6}", "-1.285600");
        assert_display!(12856, 0, true, "{:.6}", "-12856.000000");
        assert_display!(1285600, 6, false, "{}", "1.2856");
        assert_display!(u64::MAX as u128, 0, false, "{}", u64::MAX.to_string());
        assert_display!(101, -98, false, "{:.10}", "10100000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000.0000000000");
        assert_display!(101, 98, false, "{:.10}", "0.0000000000");
    }

    #[test]
    fn test_precision() {
        fn assert_precision(val: &str, expected: u8) {
            let dec = val.parse::<Decimal>().unwrap();
            assert_eq!(dec.precision(), expected);
        }

        assert_precision("0.0", 1);
        assert_precision("1", 1);
        assert_precision("10", 2);
        assert_precision("1.230", 3);
        assert_precision("123456123456", 12);
        assert_precision("123456.123456", 12);
        assert_precision("-123456.123456", 12);
        assert_precision("99999999999999999999999999999999999999", 38);
    }

    #[test]
    fn test_encoding() {
        fn assert_encoding(num: &str) {
            let num = num.parse::<Decimal>().unwrap();
            let mut buf = Vec::new();
            let size = num.compact_encode(&mut buf).unwrap();
            assert_eq!(buf.len(), size);
            let decoded_num = Decimal::decode(&buf);
            assert_eq!(decoded_num, num);
        }

        assert_encoding("0");
        assert_encoding("255");
        assert_encoding("-255");
        assert_encoding("65535");
        assert_encoding("-65535");
        assert_encoding("4294967295");
        assert_encoding("-4294967295");
        assert_encoding("18446744073709551615");
        assert_encoding("-18446744073709551615");
        assert_encoding("99999999999999999999999999999999999999");
        assert_encoding("-99999999999999999999999999999999999999");
        assert_encoding("184467440.73709551615");
        assert_encoding("-184467440.73709551615");
    }

    #[test]
    fn test_cmp() {
        macro_rules! assert_cmp {
            ($left: expr, $cmp: tt, $right: expr) => {{
                let l = $left.parse::<Decimal>().unwrap();
                let r = $right.parse::<Decimal>().unwrap();
                assert!(l $cmp r, "{} {} {}", l, stringify!($cmp),r);
            }};
        }

        assert_cmp!("0", ==, "0");

        assert_cmp!("-1", <, "1");
        assert_cmp!("1", >, "-1");

        assert_cmp!("1.1", ==, "1.1");
        assert_cmp!("1.2", >, "1.1");
        assert_cmp!("-1.2", <, "1.1");
        assert_cmp!("1.1", >, "-1.2");

        assert_cmp!("1", <, "1e39");
        assert_cmp!("1", >, "1e-39");
        assert_cmp!("1.0e-100", >=, "1.0e-101");
        assert_cmp!("1.0e-101", <=, "1.0e-100");
        assert_cmp!("1.0e-100", !=, "1.0e-101");

        assert_cmp!("1.12", <, "1.2");
        assert_cmp!("1.2", >, "1.12");
        assert_cmp!("-1.2", <, "-1.12");
        assert_cmp!("-1.12", >, "-1.2");
        assert_cmp!("-1.12", <, "1.2");
        assert_cmp!("1.12", >, "-1.2");

        assert_cmp!("0.000000001", <,"100000000");
        assert_cmp!("100000000", >, "0.000000001");

        assert_cmp!(
            "9999999999999999999999999999999999999.9", >, "9.9999999999999999999999999999999999999"
        );
        assert_cmp!(
            "9.9999999999999999999999999999999999999", >, "0"
        );
        assert_cmp!(
            "9.9999999999999999999999999999999999999", >, "1"
        );
        assert_cmp!(
            "-9999999999999999999999999999999999999.9", <, "-9.9999999999999999999999999999999999999"
        );
        assert_cmp!(
            "-9.9999999999999999999999999999999999999", <, "0"
        );
        assert_cmp!(
            "-9.9999999999999999999999999999999999999", <, "1"
        );
        assert_cmp!("4703178999618078116505370421100e39", >, "0");
        assert_cmp!("4703178999618078116505370421100e-39", >, "0");
        assert_cmp!("-4703178999618078116505370421100e39", <, "0");
        assert_cmp!("-4703178999618078116505370421100e-39", <, "0");
        assert_cmp!("0", <, "4703178999618078116505370421100e39");
        assert_cmp!("0", <, "4703178999618078116505370421100e-39");
        assert_cmp!("0", >, "-4703178999618078116505370421100e39");
        assert_cmp!("0", >, "-4703178999618078116505370421100e-39");
    }

    #[test]
    fn test_abs() {
        fn assert_abs(val: &str, expected: &str) {
            let abs_val = val.parse::<Decimal>().unwrap().abs();
            let expected = expected.parse::<Decimal>().unwrap();
            assert_eq!(abs_val, expected);
        }

        assert_abs("0.0", "0");
        assert_abs("123456.123456", "123456.123456");
        assert_abs("-123456.123456", "123456.123456");
    }

    #[test]
    fn test_trunc() {
        fn assert_trunc(val: &str, scale: i16, expected: &str) {
            let decimal = val.parse::<Decimal>().unwrap().trunc(scale);
            let expected = expected.parse::<Decimal>().unwrap();
            assert_eq!(decimal, expected);
        }

        assert_trunc("0", -1, "0");
        assert_trunc("123456", 0, "123456");
        assert_trunc("123456.123456", 6, "123456.123456");
        assert_trunc("123456.123456", 5, "123456.12345");
        assert_trunc("123456.123456", 4, "123456.1234");
        assert_trunc("123456.123456", 3, "123456.123");
        assert_trunc("123456.123456", 2, "123456.12");
        assert_trunc("123456.123456", 1, "123456.1");
        assert_trunc("123456.123456", 0, "123456");
        assert_trunc("123456.123456", -1, "123450");
        assert_trunc("123456.123456", -2, "123400");
        assert_trunc("123456.123456", -3, "123000");
        assert_trunc("123456.123456", -4, "120000");
        assert_trunc("123456.123456", -5, "100000");
        assert_trunc("9999.9", 1, "9999.9");
        assert_trunc("9999.9", -2, "9900");
        assert_trunc("9999.9", -4, "0");
        assert_trunc("1e125", 0, "1e125");
        assert_trunc("1e125", -125, "1e125");
        assert_trunc("1e-130", 0, "0");
        assert_trunc("1.7976931348623279769313486232797693134E-130", 131, "1.7E-130");
        assert_trunc(
            "1.7976931348623279769313486232797693134E-130",
            166,
            "1.797693134862327976931348623279769313E-130",
        );
        assert_trunc(
            "1.7976931348623279769313486232797693134E-130",
            167,
            "1.7976931348623279769313486232797693134E-130",
        );
        assert_trunc(
            "1.7976931348623279769313486232797693134E-130",
            168,
            "1.7976931348623279769313486232797693134E-130",
        );
    }

    #[test]
    fn test_round() {
        fn assert_round(val: &str, scale: i16, expected: &str) {
            let decimal = val.parse::<Decimal>().unwrap().round(scale);
            let expected = expected.parse::<Decimal>().unwrap();
            assert_eq!(decimal, expected);
        }

        assert_round("0", -1, "0");
        assert_round("123456", 0, "123456");
        assert_round("123456.123456", 6, "123456.123456");
        assert_round("123456.123456", 5, "123456.12346");
        assert_round("123456.123456", 4, "123456.1235");
        assert_round("123456.123456", 3, "123456.123");
        assert_round("123456.123456", 2, "123456.12");
        assert_round("123456.123456", 1, "123456.1");
        assert_round("123456.123456", 0, "123456");
        assert_round("123456.123456", -1, "123460");
        assert_round("123456.123456", -2, "123500");
        assert_round("123456.123456", -3, "123000");
        assert_round("123456.123456", -4, "120000");
        assert_round("123456.123456", -5, "100000");
        assert_round("9999.9", 1, "9999.9");
        assert_round("9999.9", -2, "10000");
        assert_round("9999.9", -4, "10000");
        assert_round("1.7976931348623279769313486232797693134E-130", 131, "1.8E-130");
        assert_round(
            "1.7976931348623279769313486232797693134E-130",
            166,
            "1.797693134862327976931348623279769313E-130",
        );
        assert_round(
            "1.7976931348623279769313486232797693134E-130",
            167,
            "1.7976931348623279769313486232797693134E-130",
        );
        assert_round(
            "1.7976931348623279769313486232797693134E-130",
            168,
            "1.7976931348623279769313486232797693134E-130",
        );
    }

    #[test]
    fn test_round_with_precision() {
        fn assert(val: &str, precision: u8, scale: i16, expected: &str) {
            let mut decimal = val.parse::<Decimal>().unwrap();
            let overflowed = decimal.round_with_precision(precision, scale);
            assert!(!overflowed);
            let expected = expected.parse::<Decimal>().unwrap();
            assert_eq!(decimal, expected);
        }

        fn assert_overflow(val: &str, precision: u8, scale: i16) {
            let mut decimal = val.parse::<Decimal>().unwrap();
            let overflowed = decimal.round_with_precision(precision, scale);
            assert!(overflowed);
        }

        assert_overflow("123456", 5, 0);
        assert_overflow("123456", 5, 1);
        assert_overflow("123456", 6, 1);
        assert_overflow("123.456", 6, 4);
        assert_overflow("5e100", 5, -2);
        assert_overflow("5e100", 20, -80);

        assert("123456", 5, -1, "123460");
        assert("123456", 5, -5, "100000");
        assert("123456", 5, -6, "0");
        assert("123456", 6, 0, "123456");
        assert("123456", 6, -1, "123460");
        assert("123.456", 6, 0, "123");
        assert("123.456", 6, 1, "123.5");
        assert("123.456", 6, 3, "123.456");
        assert("123.456", 6, -1, "120");
        assert("123.456", 6, -2, "100");
        assert("123.456", 6, -3, "0");
        assert("623.456", 6, -3, "1000");
        assert("123.456", 6, -4, "0");
        assert("123.456", 5, -4, "0");
        assert("123.456", 5, -3, "0");
        assert("123.456", 5, -2, "100");
        assert("123456", 5, -5, "100000");
        assert("123456", 5, -6, "0");
        assert("123456", 5, -7, "0");
        assert("5e100", 21, -80, "5e100");
        assert("5E-130", 10, 5, "0");
        assert("5E-47", 1, 10, "0");
        assert("-1E-130", 38, 10, "0");
        assert("0.000811111", 5, 3, "0.001");
    }

    #[test]
    fn test_normalize_to() {
        fn assert_normalize(val: (u128, i16), scale: i16, expected: (u128, i16)) {
            let left = Decimal::from_parts(val.0, val.1, false).unwrap();
            let right = Decimal::from_parts(expected.0, expected.1, false).unwrap();
            assert_eq!(left, right);
            let normal = left.normalize_to_scale(scale);
            assert_eq!((normal.int_val, normal.scale), expected);
        }

        assert_normalize((12300, MAX_SCALE), 2, (123, MAX_SCALE - 2));
        assert_normalize((12300, 2), 2, (12300, 2));
        assert_normalize((12300, 2), 3, (123000, 3));
        assert_normalize((12300, 2), 0, (123, 0));
        assert_normalize((12300, 2), -1, (123, 0));
        assert_normalize((123000, 2), -1, (123, -1));
        assert_normalize(
            (9_9999_9999_9999_9999_9999_9999_9999_9999_9999_u128, -2),
            2,
            (99_9999_9999_9999_9999_9999_9999_9999_9999_9990_u128, -1),
        );
        assert_normalize((12300, MIN_SCALE + 1), -100, (123000000000000000000000000000, -100));
    }

    #[test]
    fn test_normalize() {
        fn assert_normalize(val: (u128, i16), expected: (u128, i16)) {
            let left = Decimal::from_parts(val.0, val.1, false).unwrap();
            let right = Decimal::from_parts(expected.0, expected.1, false).unwrap();
            assert_eq!(left, right);
            let normal = left.normalize();
            assert_eq!((normal.int_val, normal.scale), expected);
        }

        assert_normalize((12300, MAX_SCALE), (123, MAX_SCALE - 2));
        assert_normalize((12300, 2), (123, 0));
        assert_normalize((1230, 0), (1230, 0));
        assert_normalize((12300, -2), (1230000, 0));
        assert_normalize(
            (9_9999_9999_9999_9999_9999_9999_9999_9999_9999_u128, -2),
            (99_9999_9999_9999_9999_9999_9999_9999_9999_9990_u128, -1),
        );
        assert_normalize((12300, MIN_SCALE + 1), (12300000000000000000000000000000000000, -92));
    }

    #[test]
    fn test_hash() {
        use std::collections::hash_map::DefaultHasher;

        let d1 = Decimal::from_parts(12345, 3, false).unwrap();
        let d2 = Decimal::from_parts(123450, 4, false).unwrap();

        let mut hash1 = DefaultHasher::new();
        let mut hash2 = DefaultHasher::new();

        d1.hash(&mut hash1);
        d2.hash(&mut hash2);

        assert_eq!(hash1.finish(), hash2.finish());
    }

    #[test]
    fn test_sqrt() {
        fn assert_sqrt(val: &str, expected: &str) {
            let num = val.parse::<Decimal>().unwrap();
            let expected = expected.parse::<Decimal>().unwrap();
            let result = num.sqrt().unwrap();
            assert_eq!(result, expected);
        }

        assert_sqrt("0", "0");
        assert_sqrt("0.00000", "0");
        assert_sqrt("1", "1");
        assert_sqrt("1.001", "1.0004998750624609648232582877001097531");
        assert_sqrt("1.44", "1.2");
        assert_sqrt("2", "1.4142135623730950488016887242096980786");
        assert_sqrt("100", "10");
        assert_sqrt("49", "7");
        assert_sqrt("0.25", "0.5");
        assert_sqrt("0.0152399025", "0.12345");
        assert_sqrt("152399025", "12345");
        assert_sqrt("0.00400", "0.063245553203367586639977870888654370675");
        assert_sqrt("0.1", "0.31622776601683793319988935444327185337");
        assert_sqrt("2", "1.4142135623730950488016887242096980786");
        assert_sqrt("125348", "354.04519485512015631084871931761013143");
        assert_sqrt(
            "18446744073709551616.1099511",
            "4294967296.0000000000127999926917254925",
        );
        assert_sqrt(
            "3.1415926535897931159979634685441851615",
            "1.7724538509055159927515191031392484393",
        );
        assert_sqrt(
            "0.000000000089793115997963468544185161590576171875",
            "0.0000094759229628550415175617837401442254225",
        );
        assert_sqrt(
            "0.71777001097629639227453423431674136248",
            "0.84721308475276536670429805177990207040",
        );
        assert_sqrt(
            "0.012345679012345679012345679012345679012",
            "0.11111111111111111111111111111111111111",
        );
        assert_sqrt(
            "0.11088900000000000000000000000000000444",
            "0.33300000000000000000000000000000000667",
        );
        assert_sqrt(
            "17014118346046923173168730371588410572",
            "4124817371235594858.7903221175243613899",
        );
        assert_sqrt(
            "0.17014118346046923173168730371588410572",
            "0.41248173712355948587903221175243613899",
        );
        assert_sqrt("1e100", "1e50");
        assert_sqrt("1.01e100", "1.0049875621120890270219264912759576187e50");
        assert_sqrt("1e-100", "1e-50");
        assert_sqrt("1.01e-100", "1.0049875621120890270219264912759576187e-50");
        assert_sqrt("1.0e-130", "1.0e-65");
    }

    #[test]
    fn test_ceil_floor() {
        fn assert_ceil_floor(val: &str, expected_ceil: &str, expected_floor: &str) {
            let decimal_ceil = val.parse::<Decimal>().unwrap().ceil();
            let decimal_floor = val.parse::<Decimal>().unwrap().floor();
            let expected_ceil = expected_ceil.parse::<Decimal>().unwrap();
            let expected_floor = expected_floor.parse::<Decimal>().unwrap();
            assert_eq!(decimal_ceil, expected_ceil);
            assert_eq!(decimal_floor, expected_floor);
        }

        assert_ceil_floor("0", "0", "0");
        assert_ceil_floor("123456", "123456", "123456");
        assert_ceil_floor("12345600", "12345600", "12345600");
        assert_ceil_floor("-12345600", "-12345600", "-12345600");
        assert_ceil_floor("123456.123456", "123457", "123456");
        assert_ceil_floor("-123456.123456", "-123456", "-123457");
        assert_ceil_floor("0.00123456", "1", "0");
        assert_ceil_floor("-0.00123456", "0", "-1");
        assert_ceil_floor("1e100", "1e100", "1e100");
        assert_ceil_floor("1e-100", "1", "0");
        assert_ceil_floor("-1e100", "-1e100", "-1e100");
        assert_ceil_floor("-1e-100", "0", "-1");
        assert_ceil_floor("100e-2", "1", "1");
        assert_ceil_floor("-100e-2", "-1", "-1");
    }

    #[test]
    fn test_simply_format() {
        fn assert_fmt(input: &str, expected: &str) {
            let mut s = String::with_capacity(256);
            let num = input.parse::<Decimal>().unwrap();
            num.simply_format(&mut s).unwrap();
            assert_eq!(s.as_str(), expected);
        }

        assert_fmt("0", "0");
        assert_fmt("0.6796000", ".6796");
        assert_fmt("0.6796", ".6796");
        assert_fmt("-0.6796", "-.6796");
        assert_fmt("123456789.123456789", "123456789.123456789");
        assert_fmt("+123456789.123456789", "123456789.123456789");
        assert_fmt("-123456789.123456789", "-123456789.123456789");
    }

    #[test]
    fn test_format_with_sci() {
        fn assert_fmt(input: &str, target_len: u16, expected: &str) {
            let mut s = String::with_capacity(256);
            let num = input.parse::<Decimal>().unwrap();
            num.format_with_sci(target_len, &mut s).unwrap();
            assert_eq!(s.as_str(), expected);
        }

        fn assert_error(input: &str, target_len: u16) {
            let mut s = String::with_capacity(256);
            let num = input.parse::<Decimal>().unwrap();
            assert!(num.format_with_sci(target_len, &mut s).is_err());
        }

        // Cannot truncates when target_len is smaller than scientific notation length
        assert_fmt("0", 1, "0");
        assert_fmt("0", 5, "0");
        assert_fmt("6", 1, "6");
        assert_fmt("6", 5, "6");
        assert_error("10", 1);
        assert_fmt("10", 2, "10");
        assert_fmt("10", 5, "10");
        assert_error("100", 2);
        assert_fmt("100", 3, "100");
        assert_fmt("100", 5, "100");
        assert_fmt("-236.23", 20, "-236.23");
        assert_fmt("-236.23", 7, "-236.23");

        // Keeps zero ending
        assert_fmt("1000000000", 10, "1000000000");
        assert_fmt("-1000000000", 11, "-1000000000");
        assert_fmt("1000000000", 9, "1.000E+09");
        assert_fmt("-1000000000", 10, "-1.000E+09");
        assert_fmt("1000000000", 7, "1.0E+09");
        assert_fmt("-1000000000", 8, "-1.0E+09");
        assert_error("1000000000", 6);
        assert_error("-1000000000", 7);

        // Rounds when truncate
        assert_fmt("9999999999", 9, "1.000E+10");
        assert_fmt("9999999999", 7, "1.0E+10");
        assert_fmt("1899999999", 9, "1.900E+09");
        assert_fmt("1899999999", 7, "1.9E+09");
        assert_fmt("1989999999", 9, "1.990E+09");
        assert_fmt("1989999999", 7, "2.0E+09");
        assert_fmt("1999999999", 9, "2.000E+09");
        assert_fmt("1999999999", 7, "2.0E+09");
        assert_fmt("1666666666", 9, "1.667E+09");
        assert_fmt("1666666666", 7, "1.7E+09");
        assert_error("1666666666", 6);
        assert_fmt("9999999999.999999999", 25, "9999999999.999999999");
        assert_fmt("9999999999.999999999", 9, "1.000E+10");
        assert_fmt("-9999999999.999999999", 9, "-1.00E+10");
        assert_fmt("666666.666666", 10, "666666.667");
        assert_fmt(".0000123456789", 10, ".000012346");
        assert_fmt(".00000123456789", 10, "1.2346E-06");
        assert_fmt(".00000999999999", 10, "1.0000E-05");
        assert_fmt("-0.00000999999999", 10, "-1.000E-05");
        assert_fmt("-0.00000999999999", 20, "-.00000999999999");
        assert_fmt("-0.0000000000123456789", 14, "-1.2345679E-11");
        assert_fmt(".0000000000123456789", 14, "1.23456789E-11");
        assert_fmt("-0.0000000000123456789", 20, "-1.2345678900000E-11");

        // Ignores zero integer
        assert_fmt("-0.0000000000123456789", 21, "-.0000000000123456789");
        assert_fmt("0.135E-100", 8, "1.4E-101");
        assert_fmt("0.135E-100", 15, "1.35000000E-101");
        assert_fmt("0.135E-100", 25, "1.350000000000000000E-101");
        assert_fmt("0.135E-100", 30, "1.35000000000000000000000E-101");
        assert_fmt("-0.135E+100", 25, "-1.350000000000000000E+99");
        assert_fmt("-0.135E+100", 30, "-1.35000000000000000000000E+99");
        assert_fmt(
            "-0.135E-100",
            106,
            "-.0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000135",
        );
        assert_fmt(
            "0.1E-126",
            127,
            "1.000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000E-127",
        );

        // Ignores ending '.' after integer
        assert_fmt("666666.666666", 7, "666667");
        assert_fmt("666666.666666", 6, "666667");
        assert_error("666666.666666", 5);

        // Ignores zeros after decimal's int_val in fraction
        fn assert_fmt2(num: Decimal, target_len: u16, expected: &str) {
            let mut s = String::with_capacity(256);
            num.format_with_sci(target_len, &mut s).unwrap();
            assert_eq!(s.as_str(), expected);
        }

        let num = Decimal::from_parts(330, 3, false).unwrap();
        assert_fmt2(num, 10, ".33");
        assert_fmt2(num, 2, ".3");
    }

    #[test]
    fn test_format_with_sci_forced() {
        fn assert_sci(input: &str, expect_scale: i16, with_zero_before_dot: bool, expect: &str) {
            let num = input.parse::<Decimal>().unwrap();
            let mut s = String::new();
            num.format_with_sci_forced(expect_scale, with_zero_before_dot, &mut s)
                .unwrap();
            assert_eq!(s.as_str(), expect);
        }

        assert_sci("0", 0, false, "0E+00");
        assert_sci("0", 1, false, " .0E+00");
        assert_sci("0", 3, false, " .000E+00");
        assert_sci(
            "0",
            56,
            false,
            " .00000000000000000000000000000000000000000000000000000000E+00",
        );
        assert_sci("0", 0, true, "0E+00");
        assert_sci("0", 1, true, "0.0E+00");
        assert_sci("0", 3, true, "0.000E+00");
        assert_sci(
            "0",
            56,
            true,
            "0.00000000000000000000000000000000000000000000000000000000E+00",
        );
        assert_sci("0.6", 0, false, "6E-01");
        assert_sci("1.6", 0, false, "2E+00");
        assert_sci("1.2", 0, false, "1E+00");
        assert_sci(
            "3.234234E120",
            56,
            false,
            "3.23423400000000000000000000000000000000000000000000000000E+120",
        );
        assert_sci(
            "3.234234E-120",
            56,
            false,
            "3.23423400000000000000000000000000000000000000000000000000E-120",
        );
        assert_sci("3.234234E120", 3, false, "3.234E+120");
        assert_sci("3.234234E-120", 3, false, "3.234E-120");
        assert_sci(
            "0.345e100",
            56,
            false,
            "3.45000000000000000000000000000000000000000000000000000000E+99",
        );
        assert_sci(
            "0.345e-100",
            56,
            false,
            "3.45000000000000000000000000000000000000000000000000000000E-101",
        );
        assert_sci("3e2", 4, false, "3.0000E+02");
        assert_sci("300", 4, false, "3.0000E+02");
        assert_sci("0.03", 4, false, "3.0000E-02");
        assert_sci("3.36e60", 0, false, "3E+60");
        assert_sci("3.36e-60", 0, false, "3E-60");
        assert_sci("-3.36e60", 0, false, "-3E+60");
        assert_sci("-3.36e-60", 0, false, "-3E-60");
        assert_sci("3.36e60", 1, false, "3.4E+60");
        assert_sci("3.36e-60", 1, false, "3.4E-60");
        assert_sci("-3.36e60", 1, false, "-3.4E+60");
        assert_sci("-3.36e-60", 1, false, "-3.4E-60");
    }

    #[test]
    fn test_pow() {
        fn assert_pow_uint(base: &str, exponent: u64, expected: &str) {
            let decimal = base.parse::<Decimal>().unwrap().pow_u64(exponent).unwrap();
            let expected = expected.parse::<Decimal>().unwrap();
            assert_eq!(decimal, expected);
        }
        fn assert_pow_int(base: &str, exponent: i64, expected: &str) {
            let decimal = base.parse::<Decimal>().unwrap().pow_i64(exponent).unwrap();
            let expected = expected.parse::<Decimal>().unwrap();
            assert_eq!(decimal, expected);
        }
        fn assert_pow_decimal(base: &str, exponent: &str, expected: &str) {
            let exponent = exponent.parse::<Decimal>().unwrap();
            let decimal = base.parse::<Decimal>().unwrap().checked_pow(&exponent).unwrap();
            let expected = expected.parse::<Decimal>().unwrap();
            assert_eq!(decimal, expected);
        }

        assert_pow_uint("0", 0, "1");
        assert_pow_uint("0", 2, "0");
        assert_pow_uint("30.03", 11, "17910538937279543.381440174900003379415");
        assert_pow_uint("0.9999999", 123456, "0.98773029366878871282374552006725694652");
        assert_pow_uint("2", 418, "676921312041214565326761275425557544830000000000000000000000000000000000000000000000000000000000000000000000000000000000000000");
        assert_pow_int("3.333", 3, "37.025927037");
        assert_pow_int("123456", -2, "0.000000000065610839816062225597621740797803625383");
        assert_pow_int("16.66666", -6, "0.000000046656111974556764327215254493713994963");
        assert_pow_int("15", -15, "0.0000000000000000022836582605211672220051325163651837732");
        assert_pow_int(
            "2",
            200,
            "1606938044258990275541962092341162602500000000000000000000000",
        );
        assert_pow_int("100", -9223372036854775808, "0");
        assert_pow_decimal("-3", "0", "1");
        assert_pow_decimal("3.333", "3", "37.025927037");
        assert_pow_decimal("3.3", "2.2", "13.827086118044145328600539201031810464");
        assert_pow_decimal("2", "50.1", "1206709641626009.0372720478765230064730");
        assert_pow_decimal("2", "-50.1", "0.00000000000000082869976795124193101335598234941507825");
        assert_pow_decimal("123456", "2.2", "158974271527.98285353227767713306007512");
        assert_pow_decimal(
            "123456",
            "-12.2",
            "0.0000000000000000000000000000000000000000000000000000000000000076480574247485409303800372083765338615",
        );
        assert_pow_decimal("123456.789", "0.9999999", "123456.64426370977396175023229704225849");
        assert_pow_decimal(
            "234567890123456.789",
            "5.8822",
            "3379043109285747020459941490972051546800000000000000000000000000000000000000000000000",
        );
        assert_pow_decimal("0.9999999", "0.789", "0.99999992109999916760496639898664270396");
        assert_pow_decimal("0.9999999", "123456.789", "0.98773021573686772017452509110356382471");
        assert_pow_decimal(
            "0.9",
            "22222220000000000000000000000000000000000000000000000000000000",
            "0",
        );
        assert_pow_decimal(
            "1",
            "22222220000000000000000000000000000000000000000000000000000000",
            "1",
        );
        assert_pow_decimal("2", "418.1", "725506298471023093722890872060236907240000000000000000000000000000000000000000000000000000000000000000000000000000000000000000");
        assert_pow_decimal(
            "1.0000000000000000000000000000000000001",
            "340282366920938463463374607431768211450",
            "600171577097065.40413095725314413792835",
        );
        assert_pow_decimal("100", "-170141183460469231731687303715884105720", "0");
        assert_pow_decimal("5", "-4188888888888888888444444444444444000000000000000000000000", "0");
    }

    #[test]
    fn test_ln() {
        fn assert_ln(val: &str, expected: &str) {
            let decimal = val.parse::<Decimal>().unwrap().ln().unwrap();
            let expected = expected.parse::<Decimal>().unwrap();
            assert_eq!(decimal, expected);
        }

        assert_ln(
            "1.0000000000000000000000000000000000001",
            "0.000000000000000000000000000000000000099999999999999999999999999999999999996",
        );
        assert_ln("0.000123456789", "-8.9996193497605301750219641082491662814");
        assert_ln("13.3", "2.5877640352277080810963887206466690594");
        assert_ln("1000", "6.9077552789821370520539743640530926228");
        assert_ln("12345.67891", "9.4210613950018353041649175905084849130");
        assert_ln("1500000000000000", "34.944241503018849642247884935729812251");
        assert_ln(
            "1500000000000000000000000000000.123456",
            "69.483017897929534902517756755995357669",
        );
        assert_ln(
            "15000000000000000000000000000000000000000000000000000000000000000000000000000",
            "175.40193217565563636734536367147602892",
        );
    }

    #[test]
    fn test_exp() {
        fn assert_exp(exponent: &str, expected: &str) {
            let decimal = exponent.parse::<Decimal>().unwrap().exp().unwrap();
            let expected = expected.parse::<Decimal>().unwrap();
            assert_eq!(decimal, expected);
        }

        assert_exp("1", "2.7182818284590452353602874713526624975");
        assert_exp("0.00000012", "1.0000001200000072000002880000086400002");
        assert_exp(
            "0.9999999999999999999999999999999999999",
            "2.7182818284590452353602874713526624971",
        );
        assert_exp("-0.00000012", "0.99999988000000719999971200000863999979");
        assert_exp(
            "-0.9999999999999999999999999999999999999",
            "0.36787944117144232159552377016146086748",
        );
        assert_exp("12.3456789", "229964.19456908213454430507162889547155");
        assert_exp("-50.1", "0.00000000000000000000017452050324689209452230894746470912110");
        assert_exp("259.11111", "33925423113202888041488548716222730394000000000000000000000000000000000000000000000000000000000000000000000000000");
        assert_exp("290.123456", "997736847550168914657296864583252087210000000000000000000000000000000000000000000000000000000000000000000000000000000000000000");
    }

    #[test]
    fn generate_exp_array() {
        // [e^0, e^290]
        for i in 0..291 {
            let exponent = Decimal::from(i);
            let result = exponent.exp_decimal().unwrap();

            if i % 5 == 0 {
                println!("// e^{}", i);
            }
            println!(
                "unsafe {{ Decimal::from_raw_parts({}, {}, {}) }},",
                result.int_val(),
                result.scale,
                result.negative,
            );
        }
    }

    #[test]
    fn generate_exp_negative_array() {
        // e^-291
        const EXP_NEGATIVE_291: Decimal =
            unsafe { Decimal::from_raw_parts(41716298478166806118243377939293045745, 164, false) };
        // [e^-299, e^-291]
        for i in 291..300 {
            let result = EXP_NEGATIVE_291.checked_div(&NATURAL_EXP[(i - 291) as usize]).unwrap();

            if i % 5 == 0 {
                println!("// e^-{}", i);
            }
            println!(
                "unsafe {{ Decimal::from_raw_parts({}, {}, {}) }},",
                result.int_val(),
                result.scale,
                result.negative,
            );
        }
    }

    #[test]
    fn test_format_to_hex() {
        fn assert_fmt_hex(input: &str, is_capital: bool, expect: &str) {
            let mut s = String::new();
            let num = input.parse::<Decimal>().unwrap();
            num.format_to_hex(is_capital, &mut s).unwrap();
            assert_eq!(s.as_str(), expect);
        }

        assert_fmt_hex("3", true, "3");
        assert_fmt_hex("15", true, "F");
        assert_fmt_hex("15", false, "f");
        assert_fmt_hex(
            "7e75",
            true,
            "f79dc0e8c518f31eb934b4522ad36a1d39f275c35e858000000000000000000"
                .to_uppercase()
                .as_str(),
        );
        assert_fmt_hex(
            "7e75",
            false,
            "f79dc0e8c518f31eb934b4522ad36a1d39f275c35e858000000000000000000",
        );
        assert_fmt_hex(
            "6e70",
            true,
            "8b18610932ab6b2906ea3dfeaa8da073a862d7e0d800000000000000000"
                .to_uppercase()
                .as_str(),
        );
        assert_fmt_hex(
            "6e70",
            false,
            "8b18610932ab6b2906ea3dfeaa8da073a862d7e0d800000000000000000",
        );
        assert_fmt_hex("999", true, "3E7");
        assert_fmt_hex("999", false, "3e7");
        assert_fmt_hex(
            "9.93879279687e53",
            true,
            "a6067cc8b3051f61f39c31e697c47c18e3c0000000000".to_uppercase().as_str(),
        );
        assert_fmt_hex(
            "9.93879279687e53",
            false,
            "a6067cc8b3051f61f39c31e697c47c18e3c0000000000",
        );
        assert_fmt_hex(
            "12345678901234567890123456789012345678e30",
            true,
            "753aaed77fe1aa5508b3e1db763b1a087e44a76fa433d81f80000000"
                .to_uppercase()
                .as_str(),
        );
        assert_fmt_hex(
            "12345678901234567890123456789012345678e30",
            false,
            "753aaed77fe1aa5508b3e1db763b1a087e44a76fa433d81f80000000",
        );
        assert_fmt_hex("253.658", true, "FE");
        assert_fmt_hex("253.658", false, "fe");
        assert_fmt_hex("0", true, "0");
        assert_fmt_hex("0", false, "0");
        assert_fmt_hex("0.2", true, "0");
        assert_fmt_hex("0.2", false, "0");
        assert_fmt_hex("0.7", true, "1");
        assert_fmt_hex("0.7", false, "1");
        // Max value
        assert_fmt_hex(
            "72370055773322622139731865630429942408e38",
            true,
            "fffffffffffffffffffffffffffffffe9e6c3ef3908c56c58cab20000000000"
                .to_uppercase()
                .as_str(),
        );
        assert_fmt_hex(
            "72370055773322622139731865630429942408e38",
            false,
            "fffffffffffffffffffffffffffffffe9e6c3ef3908c56c58cab20000000000",
        );
    }

    #[test]
    fn test_format_to_json() {
        fn assert_fmt_json(input: &str, expect: &str) {
            let mut s = String::new();
            let num = input.parse::<Decimal>().unwrap();
            num.format_to_json(&mut s).unwrap();
            assert_eq!(s.as_str(), expect);
        }

        assert_fmt_json("0", "0");
        assert_fmt_json("123", "123");
        assert_fmt_json("123.123", "123.123");
        assert_fmt_json("-123", "-123");
        assert_fmt_json("-123.123", "-123.123");
        assert_fmt_json("123e37", "1230000000000000000000000000000000000000");
        assert_fmt_json("123e38", "1.23E+40");
        assert_fmt_json("123e39", "1.23E+41");
        assert_fmt_json("12300e35", "1230000000000000000000000000000000000000");
        assert_fmt_json("12300e36", "1.23E+40");
        assert_fmt_json("12300e37", "1.23E+41");
        assert_fmt_json("-123e37", "-1230000000000000000000000000000000000000");
        assert_fmt_json("-123e38", "-1.23E+40");
        assert_fmt_json("-123e39", "-1.23E+41");
        assert_fmt_json("-12300e35", "-1230000000000000000000000000000000000000");
        assert_fmt_json("-12300e36", "-1.23E+40");
        assert_fmt_json("-12300e37", "-1.23E+41");

        assert_fmt_json("123e-42", "1.23E-40");
        assert_fmt_json("123e-41", "1.23E-39");
        assert_fmt_json("123e-40", "0.0000000000000000000000000000000000000123");
        assert_fmt_json("12300e-44", "1.23E-40");
        assert_fmt_json("12300e-43", "1.23E-39");
        assert_fmt_json("12300e-42", "0.0000000000000000000000000000000000000123");
        assert_fmt_json("-123e-42", "-1.23E-40");
        assert_fmt_json("-123e-41", "-1.23E-39");
        assert_fmt_json("-123e-40", "-0.0000000000000000000000000000000000000123");
        assert_fmt_json("-12300e-44", "-1.23E-40");
        assert_fmt_json("-12300e-43", "-1.23E-39");
        assert_fmt_json("-12300e-42", "-0.0000000000000000000000000000000000000123");

        assert_fmt_json("1234.1234e36", "1234123400000000000000000000000000000000");
        assert_fmt_json("1234.1234e37", "1.2341234E+40");
        assert_fmt_json("1234.1234e-36", "0.0000000000000000000000000000000012341234");
        assert_fmt_json("1234.1234e-37", "1.2341234E-34");

        assert_fmt_json(
            "12345678901234567890123456789012345678e2",
            "1234567890123456789012345678901234567800",
        );
        assert_fmt_json(
            "12345678901234567890123456789012345678e3",
            "1.2345678901234567890123456789012345678E+40",
        );
        assert_fmt_json(
            "12345678901234567890123456789012345678e-40",
            "0.0012345678901234567890123456789012345678",
        );
        assert_fmt_json(
            "12345678901234567890123456789012345678e-41",
            "1.2345678901234567890123456789012345678E-4",
        );

        assert_fmt_json(
            "1234567890123456789012345678901234567800e0",
            "1234567890123456789012345678901234567800",
        );
        assert_fmt_json(
            "1234567890123456789012345678901234567800e1",
            "1.2345678901234567890123456789012345678E+40",
        );
        assert_fmt_json(
            "1234567890123456789012345678901234567800e-42",
            "0.0012345678901234567890123456789012345678",
        );
        assert_fmt_json(
            "1234567890123456789012345678901234567800e-43",
            "1.2345678901234567890123456789012345678E-4",
        );

        assert_fmt_json(
            "12345678901234567.890123456789012345678e19",
            "123456789012345678901234567890123456.78",
        );
        assert_fmt_json(
            "12345678901234567.890123456789012345678e21",
            "12345678901234567890123456789012345678",
        );
        assert_fmt_json(
            "12345678901234567.890123456789012345678e23",
            "1234567890123456789012345678901234567800",
        );
        assert_fmt_json(
            "12345678901234567.890123456789012345678e24",
            "1.2345678901234567890123456789012345678E+40",
        );
        assert_fmt_json(
            "12345678901234567.890123456789012345678e-15",
            "12.345678901234567890123456789012345678",
        );
        assert_fmt_json(
            "12345678901234567.890123456789012345678e-17",
            "0.12345678901234567890123456789012345678",
        );
        assert_fmt_json(
            "12345678901234567.890123456789012345678e-19",
            "0.0012345678901234567890123456789012345678",
        );
        assert_fmt_json(
            "12345678901234567.890123456789012345678e-21",
            "1.2345678901234567890123456789012345678E-5",
        );

        assert_fmt_json(
            "0.00000000012345678901234567890123456789012345678e-1",
            "1.2345678901234567890123456789012345678E-11",
        );
        assert_fmt_json(
            "0.00000000012345678901234567890123456789012345678e0",
            "1.2345678901234567890123456789012345678E-10",
        );
        assert_fmt_json(
            "0.00000000012345678901234567890123456789012345678e6",
            "1.2345678901234567890123456789012345678E-4",
        );
        assert_fmt_json(
            "0.00000000012345678901234567890123456789012345678e7",
            "0.0012345678901234567890123456789012345678",
        );
        assert_fmt_json(
            "0.00000000012345678901234567890123456789012345678e47",
            "12345678901234567890123456789012345678",
        );
        assert_fmt_json(
            "0.00000000012345678901234567890123456789012345678e49",
            "1234567890123456789012345678901234567800",
        );
        assert_fmt_json(
            "0.00000000012345678901234567890123456789012345678e50",
            "1.2345678901234567890123456789012345678E+40",
        );
    }
}
