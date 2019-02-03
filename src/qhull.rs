use crate::iterators::{VertexIter,SetIter,VecBuffer,FacetIter};

use std::ffi::CString;
use std::os::raw::{c_int};

use crate::{qhT, qh_zero, qh_new_qhull, FILE, facetT, vertexT, qh_nearvertex, qh_memfree, qh_facetcenter, setT, qh_pointid, qh_eachvoronoi_all, qh_RIDGE_qh_RIDGEall,qh_order_vertexneighbors,qsort,qh_setsize,qh_memfreeshort, qh_ALL, qh_freeqhull, qh_compare_facetvisit};
use ndarray::{Array1, Array2};
use itertools::Itertools;


pub enum QhullMode {
    Voronoi

}

pub enum QhullOption {
    ScaleLast,
    KeepCoplanar,
    AddAPointAtInfinity,
    ExactPreMerges
}


use std::fmt::{Formatter, Error};

impl QhullOption {
    fn as_string(&self) -> String {
        match self {
            QhullOption::ScaleLast => "Qbb",
            QhullOption::KeepCoplanar => "Qc",
            QhullOption::AddAPointAtInfinity => "Qz",
            QhullOption::ExactPreMerges => "Qx"
        }.to_owned()
    }
}

impl std::fmt::Display for QhullOption {


    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        write!(f,"{}",self.as_string())
    }


}


extern { fn err() -> *mut FILE;}

pub struct QHull<'a>{
    qh: *mut qhT,
    points:  &'a mut Array2<f64>
}

#[derive(Debug)]
pub struct QhullError{
    pub external_fuction:String,
    pub error_code:c_int
}

impl QhullError {

    fn new(ef: &str, ec:c_int) -> QhullError {
        QhullError {
            external_fuction: ef.to_owned(),
            error_code: ec
        }
    }

}

struct VoronoiData {

    nridges: c_int,
    ridge_points: Vec<[i32;2]>,
    ridge_vertices: Vec<Vec<i32>>

}

use std::mem;


impl<'a> QHull<'a> {


    fn new_qht(mode:QhullMode, points: &mut Array2<f64>, options: Vec<QhullOption>) -> Result<*mut qhT, QhullError> {
        let qh_ptr: *mut qhT = unsafe {libc::malloc(mem::size_of::<qhT>() as libc::size_t)} as *mut qhT;
        if qh_ptr.is_null() {
            panic!("failed to allocate memory");
        }
        unsafe {
            qh_zero(qh_ptr, err());
        }

        let final_options = QHull::qhull_options(mode, options);

        unsafe {
            let exit_code =qh_new_qhull(qh_ptr,
                                        points.shape()[1] as c_int,
                                        points.shape()[0] as c_int,
                                        points.as_mut_ptr(),
                                        0,
                                        final_options.into_raw(),
                                        std::ptr::null_mut::<FILE>(),
                                        err());
            if exit_code != 0 {
                Err(QhullError::new("qh_new_qhull",exit_code))?;
            }


            Ok(qh_ptr)
        }

    }

    pub fn new(mode:QhullMode, points: &mut Array2<f64>, options: Vec<QhullOption>) -> Result<QHull,QhullError> {
        QHull::validatePoints(points);

        let qh_ptr = QHull::new_qht(mode, points, options)?;

        Ok(QHull{qh: qh_ptr, points})


    }

    fn validatePoints(points: &Array2<f64>) {
        if points.shape()[0] <= 0 {
            panic!("no points given");
        }
        if points.shape()[1] < 2 {
            panic!("points  need to be at least 2D. current amount of dimension is {}", points.shape()[1])
        }
    }

    fn qhull_options(mode: QhullMode, options: Vec<QhullOption>) -> CString {

        let mode_option = match mode {
            QhullMode::Voronoi => "v"

        };

        let option_str = options.into_iter().map(|o| o.as_string()).join(" ");
        CString::new(format!("qhull {} {}", mode_option, option_str)).unwrap()

    }



    pub fn get_voronoi_diagram(&self) -> (Vec<[f64; 2]>, Vec<[i32; 2]>, Vec<Vec<i32>>, Vec<Vec<i32>>, Array1<i32>) {
        let mut dist:f64 = 0.0;

        let data = self.visit_all_voronoi();


        let  (regions, mut point_region) = self.calculate_regions();


        let mut buffer = VecBuffer::new(10);


        for facet in self.facet_list().filter(|f| f.visitid > 0) {
            let center = self.facet_center(facet);
            buffer.write(center, facet.visitid);
            self.drop_center(center);
            let point_iter = SetIter::<f64>::new(facet.coplanarset).map(|f| {
                let p: *mut f64 = f;
                unsafe {std::slice::from_raw_parts_mut(p,2)}
            });
            for point in point_iter {
                let i = self.point_id(point.as_mut_ptr()) as usize;
                if i < self.points.shape()[0] {
                    let vertex = self.nearvertex(&mut dist, facet, point);
                    let j = self.point_id(vertex.point)  as usize;
                    point_region[i] = point_region[j];
                }

            }
        }

        (buffer.to_vec(), data.ridge_points, data.ridge_vertices, regions, point_region)



    }

    fn nearvertex(&self,  dist: &mut f64, facet: &mut facetT, point: &mut[f64]) ->  &vertexT {
        unsafe { &*(qh_nearvertex(self.qh, facet, point.as_mut_ptr(), dist as *mut f64)) }
    }

    fn drop_center(&self, center: &[f64]) {
        unsafe { qh_memfree(self.qh, center.as_ptr() as *mut ::std::os::raw::c_void, (*self.qh).center_size) };
    }

    fn facet_center(&self, facet: &mut facetT) -> &[f64] {
        unsafe { std::slice::from_raw_parts(qh_facetcenter(self.qh, facet.vertices),2)}

    }

    fn facet_list(&self) -> FacetIter {
        FacetIter::new(unsafe {(*self.qh).facet_list})
    }

    fn calculate_regions(&self) -> (Vec<Vec<i32>>,  Array1<c_int>) {
        self.vertex_list().
            fold((Vec::new(), self.new_point_region()), |(mut rs, mut pr), vertex| {
                self.order_vertex_neightbors(vertex);
                let i = self.point_id(vertex.point);
                if i < pr.shape()[0] as c_int {
                    pr[i as usize] = rs.len() as c_int;
                }

                rs.push(QHull::current_region(vertex));
                (rs, pr)
            })
    }

    fn new_point_region(&self) -> Array1<c_int> {
        let mut point_region = unsafe { Array1::<c_int>::uninitialized(self.points.shape()[0]) };
        point_region.fill(-1);
        point_region
    }

    fn current_region(vertex: &mut vertexT) -> Vec<i32> {
        let mut inf_seen = false;
        let mut inf_seen = false;
        let current_region: Vec<i32> =  QHull::<'a>::facet_iter(vertex.neighbors).map(
            |neighbor| (neighbor.visitid as c_int - 1) as c_int
            ).filter(move |i|
                {eprintln!("i={} inf_seen={}",i,inf_seen);
                   match (i, inf_seen) {
                    (-1,false) => {
                        inf_seen = true;
                        true
                    },
                    (-1,true) => false,
                    (_, __)    => true

                }}).peeking_take_while().collect();
        if current_region.len() == 1 && *current_region.first().unwrap() == -1 {
            eprintln!("only 1 inf in regions returning empty");
            Vec::<i32>::new()
        } else {
            current_region
        }

    }

    fn facet_iter(facet_set: *mut setT) -> SetIter<'a,facetT> {
        SetIter::<facetT>::new(facet_set)

    }

    fn point_id(&self, point: *mut f64) -> c_int {
        unsafe{qh_pointid(self.qh, point)}
    }

    fn order_vertex_neightbors(&self, vertex: &mut vertexT) {
        QHull::qh_order_vertexneighbors_nd(self.qh, (self.points.shape()[1] + 1) as c_int, vertex);
    }

    fn vertex_list(&'a self) -> VertexIter<'a> {

        VertexIter::new(unsafe{(*self.qh).vertex_list})
    }

    fn visit_all_voronoi(&self) -> VoronoiData {
        let mut data = QHull::new_vornoidata();
        unsafe {
            qh_eachvoronoi_all(self.qh, &mut data as *mut VoronoiData as *mut FILE, Some(QHull::visit_voronoi), (*self.qh).UPPERdelaunay, qh_RIDGE_qh_RIDGEall, 1);
        }
        data
    }

    fn new_vornoidata() -> VoronoiData {
        VoronoiData {
            nridges: 0,
            ridge_points: Vec::with_capacity(10),
            ridge_vertices: Vec::new()
        }
    }


    fn qh_order_vertexneighbors_nd(qh: *mut qhT, nd: c_int,  vertex: *mut vertexT) {
        if nd == 3 {
            unsafe {qh_order_vertexneighbors(qh, vertex)}
        } else if nd >= 4 {
            unsafe {
                qsort((*(*vertex).neighbors).e[0].p, qh_setsize(qh, (*vertex).neighbors) as usize,
                      mem::size_of::<facetT>(), Some(qh_compare_facetvisit))
            }


        }



    }





    unsafe extern "C" fn visit_voronoi(qh: *mut qhT,
                                       fp: *mut FILE,
                                       vertex: *mut vertexT,
                                       vertexA: *mut vertexT,
                                       centers: *mut setT,
                                       _: ::std::os::raw::c_uint) {

        let data = &mut *(fp as *mut VoronoiData);

        let point1 = qh_pointid(qh, (*vertex).point);
        let point2 = qh_pointid(qh, (*vertexA).point);

        data.ridge_points.push([point1,point2]);

        let current_vertices = QHull::<'a>::facet_iter(centers).map(|f| f.visitid as i32 -1).collect::<Vec<i32>>();

        data.ridge_vertices.push(current_vertices);

        data.nridges += 1;

    }

}


impl<'a> Drop for QHull<'a> {
    fn drop(&mut self) {
        if !self.qh.is_null() {
            let mut curlong: ::std::os::raw::c_int = 1;
            let mut totlong: ::std::os::raw::c_int = 1;
            unsafe {
                qh_freeqhull(self.qh, qh_ALL);
                qh_memfreeshort(self.qh, &mut curlong, &mut totlong);

                libc::free(self.qh as *mut libc::c_void);
            }


        }

    }
}


