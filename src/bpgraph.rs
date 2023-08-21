#[cfg(feature = "progress_output")]
use std::io::{self, Write};

use std::collections::HashMap;
use std::default::Default;
use std::fmt::Debug;
use std::sync::{Arc, Mutex};
use std::thread;

use crate::{BPError, BPResult, Msg, Node, NodeFunction, Probability};

pub type NodeIndex = usize;

pub struct BPGraph<T, MsgT: Msg<T>, CtrlMsgT = (), CtrlMsgAT: Default = ()>
where
    T: Debug,
{
    check_nodes: Vec<Node<T, MsgT, CtrlMsgT, CtrlMsgAT>>,
    var_nodes: Vec<Node<T, MsgT, CtrlMsgT, CtrlMsgAT>>,
    step: usize,
    normalize: bool,
    check_validity: bool,
    num_var_nodes: usize,
    num_check_nodes: usize,
}

impl<T, MsgT: Msg<T>, CtrlMsgT, CtrlMsgAT: Default> BPGraph<T, MsgT, CtrlMsgT, CtrlMsgAT>
where
    T: Debug,
    MsgT: Clone,
{
    /* pub fn initialize_node_constant_msg(
        &mut self,
        node_index: NodeIndex,
        msg: MsgT,
    ) -> BPResult<()> {
        let n = self.get_node_mut(node_index)?;
        for m_i in n.get_connections().clone() {
            n.send_post(m_i, msg.clone());
        }
        n.initialize()?;
        Ok(())
    } */
}

impl<T, MsgT: Msg<T> + Clone, CtrlMsgT, CtrlMsgAT: Default> BPGraph<T, MsgT, CtrlMsgT, CtrlMsgAT>
where
    T: Copy + Eq + Debug + std::hash::Hash,
    MsgT: Clone,
{
    /* pub fn get_result(
        &self,
        node_index: NodeIndex,
    ) -> BPResult<Option<std::collections::HashMap<T, Probability>>> {
        let n = self.get_node(node_index)?;
        n.get_result().map_err(|e| {
            e.attach_info_str(
                "BPGraph::get_result",
                format!("Failed to retrieve result from node {}", node_index),
            )
        })
    } */
}

impl<T, MsgT: Msg<T>, CtrlMsgT, CtrlMsgAT: Default> BPGraph<T, MsgT, CtrlMsgT, CtrlMsgAT>
where
    T: Clone + Debug,
    MsgT: Clone,
{
    /* pub fn get_inbox(&self, node_index: NodeIndex) -> BPResult<Vec<(NodeIndex, MsgT)>> {
        let node = self.get_node(node_index)?;
        Ok(node.clone_inbox())
    } */
}

impl<T, MsgT: Msg<T>, CtrlMsgT, CtrlMsgAT: Default> BPGraph<T, MsgT, CtrlMsgT, CtrlMsgAT>
where
    T: Debug,
{
    pub fn new() -> Self {
        BPGraph {
            check_nodes: Vec::new(),
            var_nodes: Vec::new(),
            step: 0,
            normalize: true,
            check_validity: false,
            num_var_nodes: 0,
            num_check_nodes: 0,
        }
    }

    pub fn set_normalize(&mut self, normalize: bool) {
        self.normalize = normalize;
    }

    /* pub fn send_control_message(
        &mut self,
        node_index: NodeIndex,
        ctrl_msg: CtrlMsgT,
    ) -> BPResult<CtrlMsgAT> {
        self.get_node_mut(node_index)?
            .send_control_message(ctrl_msg)
    } */

    pub fn set_check_validity(&mut self, value: bool) {
        self.check_validity = value;
    }

    pub fn is_initialized_check(&self) -> bool {
        self.check_nodes.iter().all(|n| n.is_initialized())
    }

    pub fn is_initialized_var(&self) -> bool {
        self.var_nodes.iter().all(|n| n.is_initialized())
    }

    /* pub fn reset(&mut self) -> BPResult<()> {
        self.check_nodes.iter_mut().try_for_each(|n| n.reset())
        self.var_nodes.iter_mut().try_for_each(|n| n.reset())
    }

    pub fn initialize_node(
        &mut self,
        node_index: NodeIndex,
        msgs: Option<Vec<(NodeIndex, MsgT)>>,
    ) -> BPResult<()> {
        if msgs.is_some() {
            self.send(vec![(node_index, msgs.unwrap())]);
        }
        let node = self.get_node_mut(node_index)?;
        node.initialize()?;
        Ok(())
    }

    pub fn initialize(&mut self) -> BPResult<()> {
        self.nodes.iter_mut().try_for_each(|node| {
            if !node.is_initialized() {
                node.initialize()
            } else {
                Ok(())
            }
        })
    }
*/
    pub fn propagate(&mut self, steps: usize) -> BPResult<()> {
        /* if !self.is_initialized() {
            return Err(BPError::new(
                "BPGraph::propagate".to_owned(),
                "Graph is not initialized".to_owned(),
            ));
        } */
        for _ in 0..steps {
            self.propagate_step()?;
        }
        Ok(())
    }

    pub fn propagate_step(&mut self) -> BPResult<()> {
        /* if self.check_validity && !self.is_valid() {
            return Err(BPError::new(
                "BPGraph::propagate_step".to_owned(),
                "Invalid graph".to_owned(),
            ));
        } */
        if self.step == 0 {
            info_print!("Initialising matrices");
            self.initialise_matrix();
        }

        info_print!("Propagating step {}", self.step);
        let num_check_nodes = self.check_nodes.len();
        for index in 0..num_check_nodes {
            //There should be a better way to iterate through check nodes
            let node = self.get_node_check(index)?;
            info_print!("Creating messages from var nodes");
            self.create_messages_var(index);
            info_print!("Creating messages in check nodes");
            self.create_messages_check(index);
            info_print!("Updating log_prob");
            self.update_log_prob(index);
        }
        info_print!("Done propagating step {}\n", self.step);
        self.step += 1;
        Ok(())
    }

    //Error messages for this
    pub fn create_messages_var(&mut self, check_node_index : usize) -> BPResult<()> {
        let variable_nodes_num = self.num_var_nodes;
        let check_node = &self.check_nodes[check_node_index];
        for var_node_index in 0..variable_nodes_num {
            let r = Node::get_matrix_value(check_node,&var_node_index);
            if let Some(var_node) = self.var_nodes.get_mut(var_node_index) { 
                let log_prob_curr: MsgT = Node::get_log_prob(var_node);
                if r.is_valid() {
                    let mut r_log: MsgT = var_node.get_log(r);
                    let mut q_log = var_node.subtract(log_prob_curr,r_log);
                    let mut q = var_node.exponent(q_log);
                    //The below function doesn't work have to change
                    var_node.update_push(check_node_index-variable_nodes_num, q);
                }   
                else {
                    let mut q = var_node.exponent(log_prob_curr);
                    var_node.update_push(check_node_index-variable_nodes_num, q);
                }
            };
        }
        Ok(())
    }

    //Error messages for this
    pub fn create_messages_check(&mut self, check_node_index : usize) -> BPResult<()> {
        if let Some(check_node) = self.check_nodes.get_mut(check_node_index) {
            let check_node_connections = check_node.get_connections();
            let mut var_messages: Vec<(usize,MsgT)> = Vec::with_capacity(check_node_connections.len());
            for var_node_index in check_node_connections {
                let var_node = &self.var_nodes[*var_node_index];
                let r = var_node.get_matrix_value(&(check_node_index).clone());
                var_messages.push((*var_node_index, (var_node.get_matrix_value(&(check_node_index)))));
            }
            //Order satisfied in this??
            let check_messages = check_node.node_function.node_function(var_messages)?;
            check_node.inbox = check_messages;
        }
        Ok(())
    }

    pub fn update_log_prob(&mut self, check_node_index : usize) -> BPResult<()>{
        if let Some(check_node) = self.check_nodes.get_mut(check_node_index) {
            let check_node_connections = check_node.get_connections();
            let variable_nodes_num = self.num_var_nodes;
            for var_node_index in check_node_connections {
                if let  Some(var_node) = self.var_nodes.get_mut(*var_node_index) {
                    let r = check_node.get_matrix_value(var_node_index);
                    let q = var_node.get_matrix_value(&(check_node_index-&variable_nodes_num));
                    var_node.update_log_prob(q,r);
                }
            }
        }
        Ok(())
    }

    //Error messages for this
    pub fn initialise_matrix(&mut self) -> () {
        for (i,node) in self.var_nodes.iter_mut().enumerate() {
            let node_connections = &node.connections;
                //Doesnt matter if there is mut here
            for node_index in node_connections {
                let initial_pdf = node.get_prior();
                node.inbox.push((*node_index, initial_pdf));
            }
        }
        for(i, node) in self.check_nodes.iter_mut().enumerate() {
            let node_connections = &node.connections;
            //Doesnt matter if there is mut here
            for node_index in node_connections {
                let zero_pdf = node.get_zero_pdf();
                node.inbox.push((*node_index, zero_pdf));
            }
        }
    }

    //Returns Node (from) -> (Node(to) -> Msg)
    /* fn create_messages(&mut self) -> BPResult<Vec<(NodeIndex, Vec<(NodeIndex, MsgT)>)>> {
        let mut res = Vec::new();
        for (i, node) in self.nodes.iter_mut().enumerate() {
            if node.is_ready(self.step)? {
                debug_print!("Creating messages at node <{}>", node.get_name());
                res.push((
                    i,
                    node.create_messages().map_err(|e| {
                        e.attach_debug_object("i", i)
                            .attach_debug_object("node.get_name()", node.get_name())
                    })?,
                ));
            }
            else {
                if node.discard_mode() {
                    node.read_post();
                }
            }
        }
        Ok(res)
    }

    //msgs: [(from, [(to, msg)])]
    fn send(&mut self, msgs: Vec<(NodeIndex, Vec<(NodeIndex, MsgT)>)>) -> BPResult<()> {
        let normalize = self.normalize;
        let check_validity = self.check_validity;
        let step = self.step;
        for (from, mut msgmap) in msgs.into_iter() {
            for (to, mut msg) in msgmap.into_iter() {
                debug_print!("Sending from {} to {}", from, to);
                let nto = self.get_node_mut(to)?;
                if !nto.get_connections().contains(&from) {
                    return Err(BPError::new(
                        "BPGraph::send".to_owned(),
                        format!(
                            "Trying to send a message along a non-existent edge ({} -> {}).",
                            from, to
                        ),
                    )
                    .attach_debug_object("step", step)
                    .attach_debug_object("edges", nto.get_connections())
                    .attach_debug_object("name of node to sending to", nto.get_name()));
                }
                if normalize {
                    msg.normalize().map_err(|e| {
                        e.attach_info_str(
                            "BPGraph::send",
                            format!("Trying to normalize message {} -> {}.", from, to),
                        )
                        .attach_debug_object("msg (the message that could not be normalized)", &msg)
                        .attach_debug_object("step", step)
                    })?;
                }
                if check_validity && !msg.is_valid() {
                    return Err(BPError::new(
                        "BPGraph::send".to_owned(),
                        format!("Trying to send an invalid message ({} -> {})", from, to),
                    )
                    .attach_debug_object("msg (the invalid message)", &msg)
                    .attach_debug_object("step", step));
                }
                nto.send_post(from, msg);
            }
        }
        Ok(())
    } */

    pub fn add_node(
        &mut self,
        name: String,
        node_function: Box<dyn NodeFunction<T, MsgT, CtrlMsgT, CtrlMsgAT> + Send + Sync>,
        is_var: bool,
    ) -> NodeIndex {
        if is_var {
            self.var_nodes.push(Node::<T, MsgT, CtrlMsgT, CtrlMsgAT>::new(
                name,
                node_function,
                is_var,
            ));
            self.num_var_nodes+=1;
            self.var_nodes.len()-1
        }
        else {
            self.check_nodes.push(Node::<T, MsgT, CtrlMsgT, CtrlMsgAT>::new(
                name,
                node_function,
                is_var,
            ));
            self.check_nodes.len()-1
        }
    }

    pub fn add_node_directly(&mut self, node: Node<T, MsgT, CtrlMsgT, CtrlMsgAT>,is_var: bool) -> NodeIndex {
        if is_var {
            self.var_nodes.push(node);
            self.num_var_nodes+=1;
            self.var_nodes.len()-1
        }
        else {
            self.check_nodes.push(node);
            self.num_check_nodes+=1;
            self.check_nodes.len()-1
        }
    }

    pub fn add_edge(&mut self, var: NodeIndex, check: NodeIndex) -> BPResult<()> {
        debug_print!("Connecting nodes {} and {}", node0, node1);
        /* if self.get_node_check(node0)?.is_factor() == self.get_node_check(node1)?.is_factor() {
            debug_print!("Cannot link nodes: {} and {}", node0, node1);
            return Err(BPError::new(
                "BPGraph::add_edge".to_owned(),
                format!(
                    "Cannot link two nodes of same type (variable/factor) ({}, {})",
                    node0, node1
                ),
            ));
        } */
        {
            let n0 = self.get_node_mut_var(var)?;
            n0.add_edge(check)?;
        }
        let n1 = self.get_node_mut_check(check)?;
        n1.add_edge(var)?;
        Ok(())
    }

    fn get_node_check(&self, node: NodeIndex) -> BPResult<&Node<T, MsgT, CtrlMsgT, CtrlMsgAT>> {
        let len = self.check_nodes.len();
        self.check_nodes.get(node).ok_or(BPError::new(
            "BPGraph::get_node".to_owned(),
            format!("Index {} out of bounds ({})", node, len),
        ))
    }

    fn get_node_var(&self, node: NodeIndex) -> BPResult<&Node<T, MsgT, CtrlMsgT, CtrlMsgAT>> {
        let len = self.var_nodes.len();
        self.var_nodes.get(node).ok_or(BPError::new(
            "BPGraph::get_node".to_owned(),
            format!("Index {} out of bounds ({})", node, len),
        ))
    }

    fn get_node_mut_check(
        &mut self,
        node: NodeIndex,
    ) -> BPResult<&mut Node<T, MsgT, CtrlMsgT, CtrlMsgAT>> {
        let len = self.check_nodes.len();
        self.check_nodes.get_mut(node).ok_or(BPError::new(
            "BPGraph::get_node".to_owned(),
            format!("Index {} out of bounds ({})", node, len),
        ))
    }

    fn get_node_mut_var(
        &mut self,
        node: NodeIndex,
    ) -> BPResult<&mut Node<T, MsgT, CtrlMsgT, CtrlMsgAT>> {
        let len = self.var_nodes.len();
        self.var_nodes.get_mut(node).ok_or(BPError::new(
            "BPGraph::get_node".to_owned(),
            format!("Index {} out of bounds ({})", node, len),
        ))
    }

    /* pub fn is_valid(&self) -> bool {
        debug_print!("Checking graph");
        self.nodes
            .iter()
            .enumerate()
            .all(|(i, _)| self.is_valid_node(i))
    }
    pub fn is_valid_node(&self, node: NodeIndex) -> bool {
        let nres = self.get_node(node);
        if nres.is_err() {
            info_print!("Could not find node {}", node);
            return false;
        }
        let n = nres.ok().unwrap();
        let cons = n.get_connections();
        if cons.is_empty() {
            info_print!("Node {} has no edges", node);
            return false;
        }
        let number_inputs = n.number_inputs();
        if number_inputs.is_some() && number_inputs.unwrap() != cons.len() {
            info_print!(
                "Node {} has a wrong number ({}) of inputs (should be: {})",
                node,
                cons.len(),
                number_inputs.unwrap()
            );
            return false;
        }
        for con in n.get_connections() {
            let nconres = self.get_node(*con);
            if nconres.is_err() {
                info_print!("Could not find node {} in connections of {}", con, node);
                return false;
            }
            let ncon = nconres.ok().unwrap();
            if !ncon.get_connections().contains(&node) {
                info_print!(
                    "{} does not have {} as connection but {} has {} as connection",
                    ncon,
                    node,
                    node,
                    ncon
                );
                return false;
            }
        }
        true
    } */
}

/* impl<T, MsgT: Msg<T>, CtrlMsgT, CtrlMsgAT: Default> std::fmt::Display
    for BPGraph<T, MsgT, CtrlMsgT, CtrlMsgAT>
where
    T: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        for (i, node) in self.nodes.iter().enumerate() {
            writeln!(f, "{}:\t{}", i, node);
        }
        writeln!(f)
    }
} */
