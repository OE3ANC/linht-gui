#!/usr/bin/env python3
# -*- coding: utf-8 -*-

#
# SPDX-License-Identifier: GPL-3.0
#
# GNU Radio Python Flow Graph
# Title: Sample M17 receiver with PTT via ZMQ
# Author: Wojciech SP5WWP / Andreas OE3ANC
# Copyright: M17 Foundation 2025
# GNU Radio version: 3.10.12.0

from gnuradio import analog
import math
from gnuradio import audio
from gnuradio import blocks
import pmt
from gnuradio import digital
from gnuradio import filter
from gnuradio.filter import firdes
from gnuradio import gr
from gnuradio.fft import window
import sys
import signal
from argparse import ArgumentParser
from gnuradio.eng_arg import eng_float, intx
from gnuradio import eng_notation
from gnuradio import m17
from gnuradio import vocoder
from gnuradio.vocoder import codec2
from gnuradio import zeromq
import numpy as np
import threading




class som_m17_ptt(gr.top_block):

    def __init__(self):
        gr.top_block.__init__(self, "Sample M17 receiver with PTT via ZMQ", catch_exceptions=True)
        self.flowgraph_started = threading.Event()

        ##################################################
        # Variables
        ##################################################
        self.samp_rate = samp_rate = int(500e3)
        self.zmq_len = zmq_len = 2048
        self.ptt = ptt = 0
        self.ch_flt = ch_flt = firdes.low_pass(1.0, samp_rate, 6.25e3, 6.25e3, window.WIN_HAMMING, 6.76)

        ##################################################
        # Blocks
        ##################################################

        self.zeromq_sub_msg_source_0 = zeromq.sub_msg_source('tcp://10.17.17.1:17001', 100, False)
        self.zeromq_pub_sink_2 = zeromq.pub_sink(gr.sizeof_short, zmq_len, 'tcp://*:17005', 100, False, (-1), '', True, True)
        self.vocoder_codec2_decode_ps_0 = vocoder.codec2_decode_ps(codec2.MODE_3200)
        self.root_raised_cosine_filter_1 = filter.interp_fir_filter_fff(
            10,
            firdes.root_raised_cosine(
                10,
                10,
                1,
                0.5,
                (8*10+1)))
        self.root_raised_cosine_filter_0 = filter.fir_filter_fff(
            1,
            firdes.root_raised_cosine(
                1,
                10,
                1,
                0.5,
                (8*10+1)))
        self.rational_resampler_xxx_1 = filter.rational_resampler_ccf(
                interpolation=500,
                decimation=48,
                taps=[],
                fractional_bw=0)
        self.rational_resampler_xxx_0 = filter.rational_resampler_ccc(
                interpolation=48,
                decimation=500,
                taps=[],
                fractional_bw=0)
        self.m17_m17_decoder_0 = m17.m17_decoder(True,False,2.0,True,False,0,'','')
        self.m17_m17_coder_0 = m17.m17_coder('OE3ANC','SP5WWP',1,1,0,0,0,0,"\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00",'','',False,False,'')
        self.freq_xlating_fir_filter_xxx_0 = filter.freq_xlating_fir_filter_ccc(1, ch_flt, 100e3, samp_rate)
        self.fir_filter_xxx_0 = filter.fir_filter_ccf(1, [0.004562185464, -0.003315159603, 0.004494660764, -0.005933215824, 0.007662642297, -0.009730151262, 0.012163462902, -0.015021624093, 0.018344736003, -0.022179571087, 0.026587935938, -0.031624350959, 0.037349001966, -0.043825719260, 0.051125585868, -0.059326237377, 0.068504390315, -0.078746186598, 0.090147725138, -0.102813489566, 0.116858526852, -0.132408962019, 0.149609118181, -0.168623734209, 0.189641904748, -0.212886091306, 0.238618491973, -0.267152027409, 0.298864858677, -0.334221364400, 0.373800820471, -0.418331953197, 0.468744199982, -0.526240588097, 0.592399581386, -0.669319857641, 0.759823522591, -0.867734659884, 0.998215908414, -1.158013367129, 1.354923496534, -1.593658456679, 1.856602265650, -2.021519762741, 1.526746272983, 1.526746272983, -2.021519762741, 1.856602265650, -1.593658456679, 1.354923496534, -1.158013367129, 0.998215908414, -0.867734659884, 0.759823522591, -0.669319857641, 0.592399581386, -0.526240588097, 0.468744199982, -0.418331953197, 0.373800820471, -0.334221364400, 0.298864858677, -0.267152027409, 0.238618491973, -0.212886091306, 0.189641904748, -0.168623734209, 0.149609118181, -0.132408962019, 0.116858526852, -0.102813489566, 0.090147725138, -0.078746186598, 0.068504390315, -0.059326237377, 0.051125585868, -0.043825719260, 0.037349001966, -0.031624350959, 0.026587935938, -0.022179571087, 0.018344736003, -0.015021624093, 0.012163462902, -0.009730151262, 0.007662642297, -0.005933215824, 0.004494660764, -0.003315159603, 0.004562185464])
        self.fir_filter_xxx_0.declare_sample_delay(0)
        self.digital_symbol_sync_xx_0 = digital.symbol_sync_ff(
            digital.TED_GARDNER,
            10,
            (2*math.pi*0.00015),
            1.0,
            1.0,
            0.005,
            1,
            digital.constellation_bpsk().base(),
            digital.IR_MMSE_8TAP,
            128,
            [])
        self.blocks_vector_source_x_0 = blocks.vector_source_b((0xDE, 0xAD, 0xBE, 0xEF, 0xCA, 0xFE, 0xBA, 0xBE, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00), True, 1, [])
        self.blocks_stream_to_vector_3 = blocks.stream_to_vector(gr.sizeof_short*1, zmq_len)
        self.blocks_stream_to_vector_2 = blocks.stream_to_vector(gr.sizeof_char*1, 64)
        self.blocks_selector_0 = blocks.selector(gr.sizeof_gr_complex*1,ptt,ptt)
        self.blocks_selector_0.set_enabled(True)
        self.blocks_packed_to_unpacked_xx_0 = blocks.packed_to_unpacked_bb(1, gr.GR_MSB_FIRST)
        self.blocks_null_sink_0 = blocks.null_sink(gr.sizeof_char*1)
        self.blocks_multiply_const_vxx_0 = blocks.multiply_const_cc((0.5 + 0j))
        self.blocks_msgpair_to_var_0 = blocks.msg_pair_to_var(self.set_ptt)
        self.blocks_message_strobe_0 = blocks.message_strobe(pmt.intern("TEST"), 10000)
        self.blocks_float_to_complex_0 = blocks.float_to_complex(1)
        self.blocks_correctiq_0 = blocks.correctiq()
        self.blocks_complex_to_float_0 = blocks.complex_to_float(1)
        self.audio_source_0 = audio.source(samp_rate, "hw:0,0", True)
        self.audio_sink_0 = audio.sink(samp_rate, 'hw:0,1', True)
        self.analog_quadrature_demod_cf_0 = analog.quadrature_demod_cf((48e3/(2.0*math.pi*800.0)))
        self.analog_frequency_modulator_fc_0 = analog.frequency_modulator_fc(((2.0*np.pi*800.0)/48e3))


        ##################################################
        # Connections
        ##################################################
        self.msg_connect((self.blocks_message_strobe_0, 'strobe'), (self.m17_m17_coder_0, 'end_of_transmission'))
        self.msg_connect((self.zeromq_sub_msg_source_0, 'out'), (self.blocks_msgpair_to_var_0, 'inpair'))
        self.connect((self.analog_frequency_modulator_fc_0, 0), (self.rational_resampler_xxx_1, 0))
        self.connect((self.analog_quadrature_demod_cf_0, 0), (self.root_raised_cosine_filter_0, 0))
        self.connect((self.audio_source_0, 1), (self.blocks_float_to_complex_0, 1))
        self.connect((self.audio_source_0, 0), (self.blocks_float_to_complex_0, 0))
        self.connect((self.blocks_complex_to_float_0, 1), (self.audio_sink_0, 1))
        self.connect((self.blocks_complex_to_float_0, 0), (self.audio_sink_0, 0))
        self.connect((self.blocks_correctiq_0, 0), (self.freq_xlating_fir_filter_xxx_0, 0))
        self.connect((self.blocks_float_to_complex_0, 0), (self.blocks_selector_0, 0))
        self.connect((self.blocks_multiply_const_vxx_0, 0), (self.blocks_selector_0, 1))
        self.connect((self.blocks_packed_to_unpacked_xx_0, 0), (self.blocks_stream_to_vector_2, 0))
        self.connect((self.blocks_selector_0, 1), (self.blocks_complex_to_float_0, 0))
        self.connect((self.blocks_selector_0, 0), (self.fir_filter_xxx_0, 0))
        self.connect((self.blocks_stream_to_vector_2, 0), (self.vocoder_codec2_decode_ps_0, 0))
        self.connect((self.blocks_stream_to_vector_3, 0), (self.zeromq_pub_sink_2, 0))
        self.connect((self.blocks_vector_source_x_0, 0), (self.m17_m17_coder_0, 0))
        self.connect((self.digital_symbol_sync_xx_0, 0), (self.m17_m17_decoder_0, 0))
        self.connect((self.fir_filter_xxx_0, 0), (self.blocks_correctiq_0, 0))
        self.connect((self.freq_xlating_fir_filter_xxx_0, 0), (self.rational_resampler_xxx_0, 0))
        self.connect((self.m17_m17_coder_0, 0), (self.root_raised_cosine_filter_1, 0))
        self.connect((self.m17_m17_decoder_0, 0), (self.blocks_null_sink_0, 0))
        self.connect((self.m17_m17_decoder_0, 0), (self.blocks_packed_to_unpacked_xx_0, 0))
        self.connect((self.rational_resampler_xxx_0, 0), (self.analog_quadrature_demod_cf_0, 0))
        self.connect((self.rational_resampler_xxx_1, 0), (self.blocks_multiply_const_vxx_0, 0))
        self.connect((self.root_raised_cosine_filter_0, 0), (self.digital_symbol_sync_xx_0, 0))
        self.connect((self.root_raised_cosine_filter_1, 0), (self.analog_frequency_modulator_fc_0, 0))
        self.connect((self.vocoder_codec2_decode_ps_0, 0), (self.blocks_stream_to_vector_3, 0))


    def get_samp_rate(self):
        return self.samp_rate

    def set_samp_rate(self, samp_rate):
        self.samp_rate = samp_rate
        self.set_ch_flt(firdes.low_pass(1.0, self.samp_rate, 6.25e3, 6.25e3, window.WIN_HAMMING, 6.76))

    def get_zmq_len(self):
        return self.zmq_len

    def set_zmq_len(self, zmq_len):
        self.zmq_len = zmq_len

    def get_ptt(self):
        return self.ptt

    def set_ptt(self, ptt):
        self.ptt = ptt
        self.blocks_selector_0.set_input_index(self.ptt)
        self.blocks_selector_0.set_output_index(self.ptt)

    def get_ch_flt(self):
        return self.ch_flt

    def set_ch_flt(self, ch_flt):
        self.ch_flt = ch_flt
        self.freq_xlating_fir_filter_xxx_0.set_taps(self.ch_flt)




def main(top_block_cls=som_m17_ptt, options=None):
    tb = top_block_cls()

    def sig_handler(sig=None, frame=None):
        tb.stop()
        tb.wait()

        sys.exit(0)

    signal.signal(signal.SIGINT, sig_handler)
    signal.signal(signal.SIGTERM, sig_handler)

    tb.start()
    tb.flowgraph_started.set()

    tb.wait()


if __name__ == '__main__':
    main()
